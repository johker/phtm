use phtm::node::execution::PushExecutor;
use phtm::node::source::Source;
use pushr::push::item::{Item, PushType};
use pushr::push::vector::{IntVector, BoolVector};
use pushr::push::graph::Graph;
use pushr::push::io::PushMessage;
use pushr::push::interpreter::{PushInterpreter, PushInterpreterState};
use pushr::push::instructions::InstructionSet;
use rand::Rng;

mod test_utils;


/// Injects a graph with a fixed number of columns. Each column node is connected to 3 cell nodes
/// (one nactive, one predictive, one active). Each of them is connected to a cell of the adjacent 
/// column via a segment node. The cell states are shifted by one in each column, so connected
/// cells typically have a different state.
pub fn inject_activate_predicted_column_graph(executor: &mut PushExecutor) {
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let tm_code = include_str!("../src/core/temporal_memory.push").to_string();
    let param_sources = Source::read_debug_code(p_code);
    let tm_sources = Source::read_debug_code(tm_code);
    executor.load(param_sources);
    executor.load(tm_sources);

    // Execute program until end of temporal memory graph creation (BP 1)
    //println!("Exec Stack = {}", executor.push_state.to_string());
    executor.step_until("BP1".to_string());

    let num_columns = test_utils::read_int_parameter(&executor, "NUM_COLUMNS".to_string()).unwrap() as usize;
    let state_cell_active = test_utils::read_int_parameter(&executor, "STATE_CELL_ACTIVE".to_string()).unwrap();
    let state_cell_inactive = test_utils::read_int_parameter(&executor, "STATE_CELL_INACTIVE".to_string()).unwrap();
    let state_cell_predictive = test_utils::read_int_parameter(&executor, "STATE_CELL_PREDICTIVE".to_string()).unwrap();
    let state_segment = test_utils::read_int_parameter(&executor, "STATE_SEGMENT".to_string()).unwrap();
    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap();

    // Inject input vector
    let mut test_input = vec![0; num_columns];
    for i in 0..num_columns >> 1 {
        test_input[i] = 1;
    }

    executor
        .push_state
        .input_stack
        .push(PushMessage::new(IntVector::new(vec![]),BoolVector::from_int_array(test_input.clone())));

    executor.step_until("BP2".to_string());

    // Inject test graph
    let n = 10;
    let mut test_graph = Graph::new();
    let mut test_col_ids = vec![0;n];
    let mut test_cell_ids = vec![0;3*n];
    let mut test_segment_ids = vec![0;3*n];


    let mut i = 0;
    while i < n {
        // Add column
        test_col_ids[i] = test_graph.add_node(state_column_active) as i32;
        let mut j = 0;
        while j < 3 {
            // Add cell node with permutation in cell state
            let ci = (i+j)%3;
            match ci {
                0 => test_cell_ids[i*3+j] = test_graph.add_node(state_cell_inactive),
                1 => test_cell_ids[i*3+j] = test_graph.add_node(state_cell_active),
                2 => test_cell_ids[i*3+j] = test_graph.add_node(state_cell_predictive),
                _ => (),
            }
            // Add connection cell to column node
            test_graph.add_edge(test_cell_ids[i*3+j], test_col_ids[i] as usize, 1.0); 
            // Add segment node
            test_segment_ids[i*3+j] = test_graph.add_node(state_segment);
            // Add connection cell to segement node
            test_graph.add_edge(test_segment_ids[i*3+j], test_cell_ids[i*3+j],1.2);
            if i > 0 {
                // Add connection previous segement node to cell
                test_graph.add_edge(test_cell_ids[i*3+j], test_segment_ids[(i-1)*3+j], 1.0);
            }
            j = j +1;
        }
        i = i +1;
    }
    // Add connection first cell to last segment 
    test_graph.add_edge(test_cell_ids[0], test_segment_ids[3*n-3],1.0);
    test_graph.add_edge(test_cell_ids[1], test_segment_ids[3*n-2],1.0);
    test_graph.add_edge(test_cell_ids[2], test_segment_ids[3*n-1],1.0);

    // Replace column_ids with injected ids
    let mut test_col_ids_item = Item::intvec(IntVector::new(test_col_ids));
    executor.push_state.name_bindings.insert("COLUMN_IDS".to_string(), test_col_ids_item);

    executor.push_state.graph_stack.push(test_graph);
}

/// Injects a graph with a fixed number of columns. Each column node is connected to 3 cell nodes
/// (one nactive, one predictive, one active). Each of them is connected to a cell of the adjacent 
/// column via a segment node. The cell states are shifted by one in each column, so connected
/// cells typically have a different state. Additionally, a fixed number of winner cells is added. 
pub fn inject_grow_synapses_graph(executor: &mut PushExecutor, found_segment: &mut bool, segment_id: &mut i32, winner_ids: &mut Vec<i32>) {
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let tm_code = include_str!("../src/core/temporal_memory.push").to_string();
    let param_sources = Source::read_debug_code(p_code);
    let tm_sources = Source::read_debug_code(tm_code);
    executor.load(param_sources);
    executor.load(tm_sources);

    // Execute program until end of temporal memory graph creation (BP 1)
    executor.step_until("BP1".to_string());

    let num_columns = test_utils::read_int_parameter(&executor, "NUM_COLUMNS".to_string()).unwrap() as usize;
    let state_cell_active = test_utils::read_int_parameter(&executor, "STATE_CELL_ACTIVE".to_string()).unwrap();
    let state_cell_inactive = test_utils::read_int_parameter(&executor, "STATE_CELL_INACTIVE".to_string()).unwrap();
    let state_cell_winner = test_utils::read_int_parameter(&executor, "STATE_CELL_WINNER".to_string()).unwrap();
    let state_cell_predictive = test_utils::read_int_parameter(&executor, "STATE_CELL_PREDICTIVE".to_string()).unwrap();
    let state_segment = test_utils::read_int_parameter(&executor, "STATE_SEGMENT".to_string()).unwrap();
    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap();

    // Inject input vector
    let mut test_input = vec![0; num_columns];
    for i in 0..num_columns >> 1 {
        test_input[i] = 1;
    }

    executor
        .push_state
        .input_stack
        .push(PushMessage::new(IntVector::new(vec![]),BoolVector::from_int_array(test_input.clone())));

    executor.step_until("BP2".to_string());

    // Inject test graph
    let mut test_graph = Graph::new();
    let mut test_col_ids = vec![0;10];
    let mut test_cell_ids = vec![0;30];
    let mut test_segment_ids = vec![0;30];


    let mut i = 0;
    while i < 10 {
        // Add column
        test_col_ids[i] = test_graph.add_node(state_column_active) as i32;
        let mut j = 0;
        while j < 3 {
            // Add cell node with permutation in cell state
            let ci = (i+j)%3;
            match ci {
                0 => test_cell_ids[i*3+j] = test_graph.add_node(state_cell_inactive),
                1 => test_cell_ids[i*3+j] = test_graph.add_node(state_cell_active),
                2 => test_cell_ids[i*3+j] = test_graph.add_node(state_cell_predictive),
                _ => (),
            }
            // Add connection cell to column node
            test_graph.add_edge(test_cell_ids[i*3+j], test_col_ids[i] as usize, 1.0); 
            // Add segment node
            test_segment_ids[i*3+j] = test_graph.add_node(state_segment);
            // Add connection cell to segement node
            test_graph.add_edge(test_segment_ids[i*3+j], test_cell_ids[i*3+j],1.2);
            if i > 0 {
                // Add connection previous segement node to cell
                test_graph.add_edge(test_cell_ids[i*3+j], test_segment_ids[(i-1)*3+j], 1.0);
            }
            j = j +1;
        }
        i = i +1;
    }
    // Add connection first cell to last segment 
    test_graph.add_edge(test_cell_ids[0], test_segment_ids[27],1.0);
    test_graph.add_edge(test_cell_ids[1], test_segment_ids[28],1.0);
    test_graph.add_edge(test_cell_ids[2], test_segment_ids[29],1.0);

    // Replace column_ids with injected ids
    let mut test_col_ids_item = Item::intvec(IntVector::new(test_col_ids));
    executor.push_state.name_bindings.insert("COLUMN_IDS".to_string(), test_col_ids_item);


    let num_winner_cell = 10;

    // Add winner cells
    for i in 0..num_winner_cell {
        let wid = test_graph.add_node(state_cell_winner);
        winner_ids.push(wid as i32);
    }

    // Find a segment cell
    for (id,node) in test_graph.nodes.iter() {
        if node.get_state() == state_segment {
           *segment_id = *id as i32;
           *found_segment = true;
           break;
        }
    }
    executor.push_state.graph_stack.push(test_graph);
}

/// Injects a single segment cell with a variable number of active potential synapses and 
/// active connected synapses
pub fn inject_test_segment(executor: &mut PushExecutor, test_segment: &mut usize, num_active_pot_sn: &mut usize, num_active_con_sn: &mut usize) {
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let tm_code = include_str!("../src/core/temporal_memory.push").to_string();
    let param_sources = Source::read_debug_code(p_code);
    let tm_sources = Source::read_debug_code(tm_code);
    executor.load(param_sources);
    executor.load(tm_sources);

    executor.step_until("BP0".to_string());
    executor.push_state.exec_stack.flush();

    let state_cell_inactive = test_utils::read_int_parameter(&executor, "STATE_CELL_INACTIVE".to_string()).unwrap();
    let state_cell_active = test_utils::read_int_parameter(&executor, "STATE_CELL_ACTIVE".to_string()).unwrap();
    let state_segment = test_utils::read_int_parameter(&executor, "STATE_SEGMENT".to_string()).unwrap();
    let connected_permanence = test_utils::read_float_parameter(&executor, "CONNECTED_PERMANENCE".to_string()).unwrap();

    let mut test_graph = Graph::new();
    *test_segment = test_graph.add_node(state_segment);
    *num_active_con_sn = 0;
    *num_active_pot_sn = 0;

    let mut rng = rand::thread_rng();
    let num_cells = rng.gen_range(15..25);
    let mut cell_ids = vec![];
    // Add cells for segment
    for i in 0..num_cells {
        // All cells are active
        cell_ids.push(test_graph.add_node(state_cell_active));
        let rnd_perm = rng.gen_range(0..100);
        if rnd_perm < 33 {
            test_graph.add_edge(cell_ids[i] as usize, *test_segment, 0.0);
        } else if rnd_perm < 66 {
            test_graph.add_edge(cell_ids[i] as usize, *test_segment, 0.8*connected_permanence);
            *num_active_pot_sn += 1;
        } else {
            test_graph.add_edge(cell_ids[i] as usize, *test_segment, 1.2*connected_permanence);
            *num_active_pot_sn += 1;
            *num_active_con_sn += 1;
        }

    }
    executor.push_state.graph_stack.push(test_graph);

}



/// Injects a single column with a fixed number of cells. Each cell is connected to a random
/// number of segments where each segment has a random number of synapses. The highest number 
/// of connected synapses and its segment id are returned.
pub fn inject_test_column(executor: &mut PushExecutor,best_matching_score: &mut i32, learning_segment_candidates: &mut Vec<i32>, winner_candidates: &mut Vec<i32>, least_used_cells: &mut Vec<i32>, winner_tmo: &mut Vec<i32>, has_matching_segments: bool){
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let tm_code = include_str!("../src/core/temporal_memory.push").to_string();
    let param_sources = Source::read_debug_code(p_code);
    let tm_sources = Source::read_debug_code(tm_code);
    executor.load(param_sources);
    executor.load(tm_sources);

    executor.step_until("BP0".to_string());

    let state_column_inactive = test_utils::read_int_parameter(&executor, "STATE_COLUMN_INACTIVE".to_string()).unwrap();
    let state_cell_inactive = test_utils::read_int_parameter(&executor, "STATE_CELL_INACTIVE".to_string()).unwrap();
    let state_cell_active = test_utils::read_int_parameter(&executor, "STATE_CELL_ACTIVE".to_string()).unwrap();
    let state_segment = test_utils::read_int_parameter(&executor, "STATE_SEGMENT".to_string()).unwrap();
    let state_cell_winner = test_utils::read_int_parameter(&executor, "STATE_CELL_WINNER".to_string()).unwrap();
    let state_segment_matching = test_utils::read_int_parameter(&executor, "STATE_SEGMENT_MATCHING".to_string()).unwrap();

    let mut test_graph = Graph::new();

    let num_cells = 4;
    let mut cell_ids = vec![];
    let mut segment_ids = vec![];
    let mut matching_segments = vec![];
    let mut least_segments_per_cell = 10;


    // Add column
    let column_id = test_graph.add_node(state_column_inactive);

    // Add cells for column
    for i in 0..num_cells {
        // Every second cell is active
        if i % 2 == 0 {
            cell_ids.push(test_graph.add_node(state_cell_active) as i32);
        } else {
            cell_ids.push(test_graph.add_node(state_cell_inactive) as i32);
        }
        test_graph.add_edge(cell_ids[i] as usize, column_id, 1.0);
        let mut rng = rand::thread_rng();
        let num_segments = rng.gen_range(2..5);
        if num_segments == least_segments_per_cell {
            least_used_cells.push(*cell_ids.get(i).unwrap());
        }
        if num_segments < least_segments_per_cell {
            least_segments_per_cell = num_segments;
            least_used_cells.clear();
            least_used_cells.push(*cell_ids.get(i).unwrap());
        }
        for j in 0..num_segments {
            let segment_id = test_graph.add_node(state_segment);
            test_graph.add_edge(segment_id, cell_ids[i] as usize, 0.5);
            segment_ids.push(segment_id);
            let num_synapses = rng.gen_range(0..10);
            // Every 2nd segment is matching
            if has_matching_segments {
                if j %2 == 0 {
                    //println!("# Syn = {}, segment_id = {}", num_synapses, segment_id);
                    if num_synapses == *best_matching_score {
                        learning_segment_candidates.push(segment_id as i32);
                        winner_candidates.push(cell_ids[i]);
                    }
                    if num_synapses > *best_matching_score {
                        //println!("New Best");
                        *best_matching_score = num_synapses as i32;
                        learning_segment_candidates.clear(); 
                        learning_segment_candidates.push(segment_id as i32);
                        winner_candidates.clear();
                        winner_candidates.push(cell_ids[i]);
                    }
                    test_graph.set_state(&segment_id, state_segment_matching);
                    matching_segments.push(segment_id as i32);
                }
            }
            for k in 0..num_synapses {
               let presynaptic_node = test_graph.add_node(state_cell_active);
               test_graph.add_edge(presynaptic_node, segment_id, 0.2);
            }
        }
    }

    // Add four cells to winner_tmo
    for i in 0..4 {
        winner_tmo.push(test_graph.add_node(state_cell_winner) as i32);
    }

    executor.push_state.name_bindings.insert("BEST_SCORE".to_string(), Item::int(0));
    executor.push_state.int_vector_stack.push(IntVector::new(cell_ids));
    executor.push_state.graph_stack.push(test_graph.clone()); // Push Graph(t-1)
    executor.push_state.graph_stack.push(test_graph); // Push Graph(t)
    executor.push_state.exec_stack.flush();

   }


#[test]
fn temporal_memory_graph_initialization() {
    let mut executor = PushExecutor::new();
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let tm_code = include_str!("../src/core/temporal_memory.push").to_string();
    let param_sources = Source::read_debug_code(p_code);
    let tm_sources = Source::read_debug_code(tm_code);
    executor.load(param_sources);
    executor.load(tm_sources);

    // Execute program until end of temporal memory graph creation (BP 1)
    executor.step_until("BP1".to_string());
    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let icache = instruction_set.cache();
    let mut i = 0;
    let mut print_on = false;

    let num_columns = test_utils::read_int_parameter(&executor, "NUM_COLUMNS".to_string()).unwrap() as usize;
    let num_cells = test_utils::read_int_parameter(&executor, "NUM_CELLS".to_string()).unwrap() as usize;
    let column_ids = test_utils::read_int_vector_parameter(&executor, "COLUMN_IDS".to_string()).unwrap();

    //println!("{}", executor.push_state.graph_stack.to_string());
    assert_eq!(column_ids.len(), num_columns);
    assert_eq!(executor.push_state.graph_stack.size(), 2);
    let graph = executor.push_state.graph_stack.get(0).unwrap();
    assert_eq!(graph.node_size(), num_columns * (num_cells + 1));
    assert_eq!(graph.edge_size(), num_columns * num_cells);
}

#[test]
fn temporal_memory_graph_activity_transfer() {
    let mut executor = PushExecutor::new();
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let tm_code = include_str!("../src/core/temporal_memory.push").to_string();
    let param_sources = Source::read_debug_code(p_code);
    let tm_sources = Source::read_debug_code(tm_code);
    executor.load(param_sources);
    executor.load(tm_sources);

    // Execute program until end of temporal memory graph creation (BP 1)
    executor.step_until("BP1".to_string());

    let num_columns = test_utils::read_int_parameter(&executor, "NUM_COLUMNS".to_string()).unwrap() as usize;
    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap();
    let state_column_inactive = test_utils::read_int_parameter(&executor, "STATE_COLUMN_INACTIVE".to_string()).unwrap();
    let num_columns = test_utils::read_int_parameter(&executor, "NUM_COLUMNS".to_string()).unwrap() as usize;
    let column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap();
    let column_inactive = test_utils::read_int_parameter(&executor, "STATE_COLUMN_INACTIVE".to_string()).unwrap();
    let num_cells = test_utils::read_int_parameter(&executor, "NUM_CELLS".to_string()).unwrap() as usize;
    let column_ids = test_utils::read_int_vector_parameter(&executor, "COLUMN_IDS".to_string()).unwrap();

    // Inject input vector
    let mut test_input = vec![0; num_columns];
    for i in 0..num_columns >> 1 {
        test_input[i] = 1;
    }

    executor
        .push_state
        .input_stack
        .push(PushMessage::new(IntVector::new(vec![]), BoolVector::from_int_array(test_input.clone())));

    //executor.step_until("BP2".to_string());
    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let icache = instruction_set.cache();
    let mut i = 0;

    loop {
        if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &icache) {
          break;
        }
        if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
            if next_instruction.to_string().starts_with("BP2") {
                break;
            }
            if executor.push_state.exec_stack.size() > 0 {
                println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
            }
            println!("INTEGER = {}", executor.push_state.int_stack.to_string());
            println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
            println!("FLOATVECTOR = {}", executor.push_state.float_vector_stack.to_string());
            println!("BOOL = {}", executor.push_state.bool_stack.to_string());
            println!("FLOAT = {}", executor.push_state.float_stack.to_string());
            println!("------------------------------------");
        }
        i += 1;
        assert!(i<20000, "Max loop counter exceeded");
    }
    let graph = executor.push_state.graph_stack.get(0).unwrap();
    let mut cidx = 0;
    for cid in column_ids {
        let columns_state = graph.get_state(&(cid as usize)).unwrap();
        assert_eq!(
            columns_state == column_active,
            test_input[cidx] == 1
        );
        assert_eq!(
            columns_state == column_inactive,
            test_input[cidx] == 0
        );
        cidx += 1;
    }
}

#[test]
fn temporal_memory_activates_predicted_column() {

    let mut executor = PushExecutor::new();
    inject_activate_predicted_column_graph(&mut executor);
    let graph_before_learning = executor.push_state.graph_stack.copy(0).unwrap();

    let state_cell_active = test_utils::read_int_parameter(&executor, "STATE_CELL_ACTIVE".to_string()).unwrap();
    let state_cell_inactive = test_utils::read_int_parameter(&executor, "STATE_CELL_INACTIVE".to_string()).unwrap();
    let state_cell_predictive = test_utils::read_int_parameter(&executor, "STATE_CELL_PREDICTIVE".to_string()).unwrap();
    let state_segment_active = test_utils::read_int_parameter(&executor, "STATE_SEGMENT_ACTIVE".to_string()).unwrap();
    let state_segment_matching_active = test_utils::read_int_parameter(&executor, "STATE_SEGMENT_ACTIVE_MATCHING".to_string()).unwrap();
    let perm_increment = test_utils::read_float_parameter(&executor, "PERM_INCREMENT".to_string()).unwrap();
    let perm_decrement = test_utils::read_float_parameter(&executor, "PERM_DECREMENT".to_string()).unwrap();
    let learning_enabled = test_utils::read_boolean_parameter(&executor, "LEARNING_ENABLED".to_string()).unwrap();

    // Execute learning step
    let mut latest_graph = graph_before_learning.clone();
    let mut previous_graph = latest_graph.clone();

    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let instruction_cache = instruction_set.cache(); 

    let mut print_on = false;
    let mut i = 0;
    loop {
        if PushInterpreter::step(
            &mut executor.push_state,
            &mut instruction_set,
            &instruction_cache,
        ) {
            break;
        }
      if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
            if next_instruction.to_string().starts_with("PRINT") {
                print_on = !print_on;
                previous_graph = latest_graph;
                latest_graph = executor.push_state.graph_stack.copy(0).unwrap();
                if let Some(diff) = previous_graph.diff(&latest_graph) {

                    println!("DIFF: {}", diff); 
                } else {
                    println!("NO DIFF");
                }
            }
            if print_on {
                println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                println!("------------------------------------");
            }
            if next_instruction.to_string() == "BP3".to_string() {
                break;
            }
        }
      i += 1;
      assert!(i<10000, "Max loop counter exceeded");
    }

    assert_eq!(learning_enabled, true);
    let graph_after_learning = executor.push_state.graph_stack.pop().unwrap();

    // All columns are active in the test graph, so any segment
    // should have a change in its synapses.
    for (segment_id, segment_node) in graph_after_learning.nodes.iter() {
        if segment_node.get_state() == state_segment_active || segment_node.get_state() == state_segment_matching_active {
            test_permanence_adjustement(&executor, &graph_before_learning, &graph_after_learning, &segment_id );
        }
    }
}



fn test_permanence_adjustement(executor: &PushExecutor, graph_before_learning: &Graph, graph_after_learning: &Graph, segment_id: &usize) {
    let state_cell_active = test_utils::read_int_parameter(&executor, "STATE_CELL_ACTIVE".to_string()).unwrap();
    let state_cell_inactive = test_utils::read_int_parameter(&executor, "STATE_CELL_INACTIVE".to_string()).unwrap();
    let perm_increment = test_utils::read_float_parameter(&executor, "PERM_INCREMENT".to_string()).unwrap();
    let perm_decrement = test_utils::read_float_parameter(&executor, "PERM_DECREMENT".to_string()).unwrap();
    println!("Found segment: {}", segment_id);
    if let Some(segment_edges) = graph_after_learning.edges.get(segment_id) {
        for se in segment_edges.iter() {
            let presyn_cell_id = se.get_origin_id();
                if graph_before_learning.get_state(&presyn_cell_id).unwrap() == state_cell_active {
                    println!("Expected Increase in Edge: {} <- {} ",segment_id.to_string(), presyn_cell_id.to_string());
                    assert_eq!(graph_before_learning.get_weight(&presyn_cell_id,&segment_id).unwrap() + perm_increment, graph_after_learning.get_weight(&presyn_cell_id,&segment_id).unwrap());
                }
                if graph_before_learning.get_state(&presyn_cell_id).unwrap() == state_cell_inactive {
                    println!("Expected Decrease in Edge: {} <- {} ",segment_id.to_string(), presyn_cell_id.to_string());
                    assert_eq!(graph_before_learning.get_weight(&presyn_cell_id,&segment_id).unwrap() - perm_decrement, graph_after_learning.get_weight(&presyn_cell_id,&segment_id).unwrap());
                }
            }
        }

}

fn test_punishment(executor: &PushExecutor, graph_before_learning: &Graph, graph_after_learning: &Graph, segment_id: &usize) {
    let state_cell_active = test_utils::read_int_parameter(&executor, "STATE_CELL_ACTIVE".to_string()).unwrap();
    let predicted_decrement = test_utils::read_float_parameter(&executor, "PREDICTED_DECREMENT".to_string()).unwrap();

    println!("Found segment: {}", segment_id);
    if let Some(segment_edges) = graph_after_learning.edges.get(segment_id) {
        for se in segment_edges.iter() {
            let presyn_cell_id = se.get_origin_id();
                if graph_before_learning.get_state(&presyn_cell_id).unwrap() == state_cell_active {
                    println!("Expected Decrease in Edge: {} <- {} ",segment_id.to_string(), presyn_cell_id.to_string());
                    assert_eq!(graph_before_learning.get_weight(&presyn_cell_id,&segment_id).unwrap() - predicted_decrement, graph_after_learning.get_weight(&presyn_cell_id,&segment_id).unwrap());
                }
            }
        }

}

#[test]
fn temporal_memory_grows_synapses() {

    let mut executor = PushExecutor::new();
    let mut segment_id = 0;
    let mut found_segment = false;
    let mut winner_ids = vec![];
    inject_grow_synapses_graph(&mut executor, &mut found_segment, &mut segment_id, &mut winner_ids);

    let test_graph = executor.push_state.graph_stack.get(0).unwrap();
    assert!(found_segment);
    println!("Chosen segment: {}", segment_id);
    let segment_synapses_before_test = test_graph.edges.get(&(segment_id as usize)).unwrap().len();

    let num_new_synapses = 5; 

    executor.push_state.int_stack.push(segment_id as i32);
    executor.push_state.int_stack.push(num_new_synapses);
    executor.push_state.int_vector_stack.push(IntVector::new(winner_ids));

    executor.push_state.exec_stack.flush();
    executor
       .push_state
       .exec_stack
       .push(Item::id("GROW_SYNAPSES".to_string()));


    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let icache = instruction_set.cache();
    let mut i = 0;
    let mut print_on = false;
    loop {
          if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &icache) {
              break;
          }
          if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
                if next_instruction.to_string().starts_with("PRINT") {
                    print_on = !print_on;
                }
                if print_on {
                    println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                    println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                    println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                    println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                    println!("------------------------------------");
                }
          }
            i += 1;
            assert!(i<10000, "Max loop counter exceeded");
    }
    let graph_after_synapse_growth = executor.push_state.graph_stack.pop().unwrap();;
    assert_eq!(graph_after_synapse_growth.edges.get(&(segment_id as usize)).unwrap().len(), segment_synapses_before_test + num_new_synapses as usize);
        assert_eq!(executor.push_state.int_stack.size(), 0);

    }

#[test]
fn temporal_memory_burst_column_with_matching_segment() {

        let mut executor = PushExecutor::new();
        let mut best_matching_score = 0;
        let mut learning_segement_candidates = vec![];
        let mut winner_candidates = vec![];
        let mut least_used_cells = vec![];
        let mut winner_tmo : Vec<i32> = vec![];

        inject_test_column(&mut executor, &mut best_matching_score, &mut learning_segement_candidates, &mut winner_candidates, &mut least_used_cells, &mut winner_tmo, true);
        executor.push_state.exec_stack.push(Item::id("BURST_COLUMN".to_string()));
        let mut instruction_set = InstructionSet::new();
        instruction_set.load();
        let icache = instruction_set.cache();
        let mut i = 0;
        let mut print_on = false;
        println!("GRAPH = {} " , executor.push_state.graph_stack.to_string());
        loop {
          if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &icache) {
              break;
          }
          if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
                if next_instruction.to_string().starts_with("PRINT") {
                    print_on = !print_on;
                }
                if print_on {
                    println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                    println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                    println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                    println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                    println!("------------------------------------");
                }
          }
            i += 1;
            assert!(i<10000, "Max loop counter exceeded");
        }
        let winner_cell = test_utils::read_int_parameter(&executor, "WINNER_CELL".to_string()).unwrap();
        let learning_segement = test_utils::read_int_parameter(&executor, "LEARNING_SEGMENT".to_string()).unwrap();
        println!("Winner Candidates = {:?}", winner_candidates);
        println!("Winner = {}", winner_cell);
        println!("Learning Candidates = {:?}", learning_segement_candidates);
        println!("Learning = {}", learning_segement);
        assert!(winner_candidates.contains(&winner_cell));
        assert!(learning_segement_candidates.contains(&learning_segement));

}

#[test]
fn temporal_memory_burst_column_without_matching_segment() {
    let mut executor = PushExecutor::new();
    let mut best_matching_score = 0;
    let mut learning_segement_candidates = vec![];
    let mut winner_candidates = vec![];
    let mut least_used_cell_candidates = vec![];
    let mut winner_tmo : Vec<i32> = vec![];

    inject_test_column(&mut executor, &mut best_matching_score, &mut learning_segement_candidates, &mut winner_candidates, &mut least_used_cell_candidates, &mut winner_tmo, false);
    executor.push_state.exec_stack.push(Item::id("BURST_COLUMN".to_string()));
    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let icache = instruction_set.cache();
    let mut i = 0;
    let mut print_on = false;
    loop {
          if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &icache) {
              break;
          }
          if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
                if next_instruction.to_string().starts_with("PRINT") {
                    print_on = !print_on;
                }
                if print_on {
                    println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                    println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                    println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                    println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                    println!("------------------------------------");
                }
          }
            i += 1;
            assert!(i<10000, "Max loop counter exceeded");
    }
    let winner_cell = test_utils::read_int_parameter(&executor, "WINNER_CELL".to_string()).unwrap();
    let learning_segement = test_utils::read_int_parameter(&executor, "LEARNING_SEGMENT".to_string()).unwrap() as usize;
    let synapse_sample_size = test_utils::read_int_parameter(&executor, "SYNAPSE_SAMPLE_SIZE".to_string()).unwrap();
    let graph = executor.push_state.graph_stack.pop().unwrap();
    println!("Least used candidates= {:?}", least_used_cell_candidates);
    println!("Winner = {}" , winner_cell);
    assert!(least_used_cell_candidates.contains(&winner_cell));
    let segment_edges = graph.edges.get(&learning_segement).unwrap();
    println!("GRAPH = {}", executor.push_state.graph_stack.get(0).unwrap());
    println!("Segment Edges = {:?}", segment_edges);
    assert_eq!(synapse_sample_size as usize, segment_edges.len());
}

#[test]
fn temporal_memory_punish_predicted_column() {
    let mut executor = PushExecutor::new();
    let mut best_matching_score = 0;
    let mut learning_segement_candidates = vec![];
    let mut winner_candidates = vec![];
    let mut least_used_cell_candidates = vec![];
    let mut winner_tmo : Vec<i32> = vec![];

    inject_test_column(&mut executor, &mut best_matching_score, &mut learning_segement_candidates, &mut winner_candidates, &mut least_used_cell_candidates, &mut winner_tmo, true);
    executor.push_state.exec_stack.push(Item::id("PUNISH_PREDICTED_COLUMN".to_string()));

    let learning_enabled = test_utils::read_boolean_parameter(&executor, "LEARNING_ENABLED".to_string()).unwrap();
    let state_segment_matching = test_utils::read_int_parameter(&executor, "STATE_SEGMENT_MATCHING".to_string()).unwrap();
    let state_segment_matching_active = test_utils::read_int_parameter(&executor, "STATE_SEGMENT_ACTIVE_MATCHING".to_string()).unwrap();

    let graph_before_learning = executor.push_state.graph_stack.copy(0).unwrap();
    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let icache = instruction_set.cache();
    let mut i = 0;
    let mut print_on = false;
    loop {
        if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &icache) {
          break;
        }
        if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
            if next_instruction.to_string().starts_with("PRINT") {
                print_on = !print_on;
            }
            if print_on {
                println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                println!("------------------------------------");
            }
        }
        i += 1;
        assert!(i<10000, "Max loop counter exceeded");
    }
    assert_eq!(learning_enabled, true);
    let graph_after_learning = executor.push_state.graph_stack.pop().unwrap();
    println!("GRAPH = {}", graph_before_learning);
    if let Some(diff) = graph_before_learning.diff(&graph_after_learning) {
        println!("Diff = {}", diff);
    }
    // All columns are active in the test graph, so any segment
    // should have a change in its synapses.
    for (segment_id, segment_node) in graph_after_learning.nodes.iter() {
        if segment_node.get_state() == state_segment_matching || segment_node.get_state() == state_segment_matching_active {
            println!("Found segment: {}, state = {}", segment_id, segment_node.get_state());
            test_punishment(&executor, &graph_before_learning, &graph_after_learning, &segment_id );
        }
    }
}


#[test]
fn temporal_memory_calculates_number_active_potential_synapses() {
    let mut executor = PushExecutor::new();
    let mut expect_act_pot_sn = 0;
    let mut expect_act_con_sn = 0;
    let mut test_segment = 0;

    inject_test_segment(&mut executor, &mut test_segment, &mut expect_act_pot_sn, &mut expect_act_con_sn);
    executor.push_state.exec_stack.push(Item::id("NUM_ACTIVE_POTENTIAL_SYNAPSES".to_string()));
    executor.push_state.int_stack.push(test_segment as i32);
    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let icache = instruction_set.cache();
    let mut i = 0;
    let mut print_on = false;

    println!("{}", executor.push_state.graph_stack.copy(0).unwrap());
    loop {
        if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &icache) {
          break;
        }
        if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
            if next_instruction.to_string().starts_with("PRINT") {
                print_on = !print_on;
            }
            if print_on {
                println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                println!("FLOAT = {}", executor.push_state.float_stack.to_string());
                println!("------------------------------------");
            }
        }
        i += 1;
        assert!(i<1000, "Max loop counter exceeded");
    }

    let num_active_pot_sn = test_utils::read_int_parameter(&executor, "NUM_ACTIVE_POTENTIAL".to_string()).unwrap() as usize;

    assert_eq!(expect_act_pot_sn, num_active_pot_sn);

}

use pushr::push::io::PushMessage;
use phtm::node::execution::PushExecutor;
use phtm::node::source::Source;
use pushr::push::item::{Item};
use pushr::push::vector::{IntVector, BoolVector};
use pushr::push::graph::Graph;
use pushr::push::interpreter::{PushInterpreter};
use pushr::push::instructions::InstructionSet;

mod test_utils;

#[test]
fn spatial_pooler_graph_initialization() {
    let mut executor = PushExecutor::new();
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let sp_code = include_str!("../src/core/spatial_pooler.push").to_string();

    let param_sources = Source::read_debug_code(p_code);
    let sp_sources = Source::read_debug_code(sp_code);

    executor.load(param_sources);
    executor.load(sp_sources);
    executor.step_until("BP0".to_string());

    let num_columns = test_utils::read_int_parameter(&executor, "NUM_COLUMNS".to_string()).unwrap() as usize;
    let num_inputs = test_utils::read_int_parameter(&executor, "NUM_INPUTS".to_string()).unwrap() as usize;
    let potential_pool_prct = test_utils::read_float_parameter(&executor, "POTENTIAL_PCT".to_string()).unwrap();

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
            if next_instruction.to_string().starts_with("BP3") {
                break;
            }
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
        assert!(i<100000, "Max loop counter exceeded");
    }
    //println!("Graph = {}", executor.push_state.graph_stack.get(0).unwrap().to_string());

    let column_ids = test_utils::read_int_vector_parameter(&executor, "COLUMN_IDS".to_string()).unwrap();
    let input_ids = test_utils::read_int_vector_parameter(&executor, "INPUT_IDS".to_string()).unwrap();
    let expected_num_connections = ((((num_inputs as f32)*potential_pool_prct).round() as usize + 2)*num_columns) as i32; // +2 for boost / overlap connection
    let connection_error_margin = (expected_num_connections as f32) * 0.1;

    assert_eq!(column_ids.len(), num_columns);
    assert_eq!(input_ids.len(), num_inputs);
    assert_eq!(executor.push_state.graph_stack.size(), 1);
    let initialized_graph = executor.push_state.graph_stack.get(0).unwrap();
    assert_eq!(initialized_graph.node_size(), 2*num_columns + num_inputs, "Two nodes for each column (column and boost node) and one for each input expected");
    assert!((i32::abs(initialized_graph.edge_size() as i32 - expected_num_connections) as f32) < connection_error_margin, "The number of edges should be about the potential pool percentage of the number of inputs"); 
    assert_eq!(executor.push_state.int_stack.size(), 0);

    let perm_con_threshold =
        test_utils::read_float_parameter(&executor, "PERM_CON_THRESHOLD".to_string()).unwrap();
    let perm_sdt_dev =
        test_utils::read_float_parameter(&executor, "PERM_STD_DEV".to_string()).unwrap();
    let state_boost_cell =
        test_utils::read_int_parameter(&executor, "STATE_BOOST_CELL".to_string()).unwrap();


    // Test of initial synaptic permanences

    let mut sum = 0.0;
    let mut count = 0.0;

    // Loop througth columns:
    for cid in column_ids.clone() {
        // Find boost cell
        let mut bid = -1;
        if let Some(column_edges) = initialized_graph.edges.get(&(cid as usize)) {
            for ce in column_edges.iter() { 
                if let Some(origin_state) = initialized_graph.get_state(&ce.get_origin_id()) {
                    if state_boost_cell == origin_state {
                        bid = ce.get_origin_id() as i32;
                        continue;
                    }
                }
            }
        }
        assert!(bid != -1, "Any column should be preceeded by a boost cell");
        assert!(initialized_graph.get_weight(&(cid as usize), &(bid as usize)) != None, "Expected edge COLUMN -> BOOST");
        assert!(initialized_graph.get_weight(&(bid as usize), &(cid as usize)) != None, "Expected edge BOOST -> COLUMN");
        if let Some(input_edges) = initialized_graph.edges.get(&(bid as usize)) {
            for ie in input_edges.iter() {
                sum += ie.get_weight();
                count += 1.0;
            }
        }
    }
    println!("sum = {}, count = {}", sum, count);
    assert!(
        f32::abs(sum / count - perm_con_threshold) < 3.0 * perm_sdt_dev,
        "The mean value of the permanence vector should be the connection threshold"
    );
}





#[test]
fn spatial_pooler_graph_topology() {
    let mut executor = PushExecutor::new();
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let sp_code = include_str!("../src/core/spatial_pooler.push").to_string();

    let param_sources = Source::read_debug_code(p_code);
    let sp_sources = Source::read_debug_code(sp_code);

    executor.load(param_sources);
    executor.load(sp_sources);
    executor.step_until("BP0".to_string());

    let num_columns = test_utils::read_int_parameter(&executor, "NUM_COLUMNS".to_string()).unwrap() as usize;
    let num_inputs = test_utils::read_int_parameter(&executor, "NUM_INPUTS".to_string()).unwrap() as usize;

    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let icache = instruction_set.cache();
    let mut i = 0;
    let mut print_on = false;
    // Test topoology 
    loop {
        if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &icache) {
          break;
        }
        if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
            if next_instruction.to_string().starts_with("BP4") {
                break;
            }
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
        assert!(i<100000, "Max loop counter exceeded");
    }

    let initialized_graph = executor.push_state.graph_stack.get(0).unwrap();
    let column_ids = test_utils::read_int_vector_parameter(&executor, "COLUMN_IDS".to_string()).unwrap();
    let state_column_inactive =
        test_utils::read_int_parameter(&executor, "STATE_COLUMN_INACTIVE".to_string()).unwrap();
    let topology_edge_length =
        test_utils::read_int_parameter(&executor, "TOPOLOGY_EDGE_LENGTH".to_string()).unwrap() as usize;

    // TOPOLOGY_EDGE_LENGTH =: N
    // NUM_COLUMNS =: n
    // Square n^2 = N

    let mut corner_nodes = 0;
    let mut edge_nodes = 0;
    let mut inner_nodes = 0;

    println!("GRAPH = {}", executor.push_state.graph_stack.get(0).unwrap().to_string());
    for cid in column_ids.clone() {
        let mut edge_count = 0;
        if let Some(column_edges) = initialized_graph.edges.get(&(cid as usize)) {
            for ce in column_edges.iter() { 
                if let Some(origin_state) = initialized_graph.get_state(&ce.get_origin_id()) {
                    // Only consider inactive columns / ignore loops
                    if state_column_inactive == origin_state && ce.get_origin_id() != cid as usize {
                        edge_count += 1;
                    }
                }
            }
        }
        match edge_count {
            2 => corner_nodes += 1,
            3 => edge_nodes += 1,
            4 => inner_nodes += 1,
            _ => {
                println!("Node ID = {}, Edge Count = {}", cid, edge_count);
                assert!(false, "Unexpected number of inbound connections from other column") 
            },
        }
    }
    println!("Corner = {}, Edge = {}, Inner = {}", corner_nodes, edge_nodes,inner_nodes);
    assert_eq!(corner_nodes, 4, "Expected 4 corner nodes with 2 inbound connections");
    assert_eq!(edge_nodes, 4*(topology_edge_length-2), "Expected 4*n-4 edge nodes with 3 inbound connections");
    assert_eq!(inner_nodes, i32::max(num_columns as i32 - (corner_nodes + edge_nodes) as i32, 0) as usize, "Expected N-4*n inner nodes with 4 inbound connections");
}


#[test]
fn spatial_pooler_calculate_overlap_to_input() {
    let mut executor = PushExecutor::new();
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let sp_code = include_str!("../src/core/spatial_pooler.push").to_string();

    let param_sources = Source::read_debug_code(p_code);
    let sp_sources = Source::read_debug_code(sp_code);

    executor.load(param_sources);
    executor.load(sp_sources);
    executor.step_until("BP4".to_string());

    let num_columns = test_utils::read_int_parameter(&executor, "NUM_COLUMNS".to_string()).unwrap() as usize;
    let num_inputs = test_utils::read_int_parameter(&executor, "NUM_INPUTS".to_string()).unwrap() as usize;
    let state_input_on = test_utils::read_int_parameter(&executor, "STATE_INPUT_ON".to_string()).unwrap();
    let state_input_off = test_utils::read_int_parameter(&executor, "STATE_INPUT_OFF".to_string()).unwrap();
    let state_boost_cell = test_utils::read_int_parameter(&executor, "STATE_BOOST_CELL".to_string()).unwrap();


    // Inject input vector
    let mut test_input = vec![1; num_inputs];
    for i in 0..num_inputs >> 1 {
        test_input[i] = 0;
    }
    test_input[1] = 0;

    executor
        .push_state
        .input_stack
        .push(PushMessage::new(IntVector::new(vec![]), BoolVector::from_int_array(test_input.clone())));

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
            if next_instruction.to_string().starts_with("BP6") {
                break;
            }
            if next_instruction.to_string().starts_with("PRINT") {
                print_on = !print_on;
            }
            if print_on {
                println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                println!("BOOLVECTOR = {}", executor.push_state.bool_vector_stack.to_string());
                println!("FLOAT = {}", executor.push_state.float_stack.to_string());
                println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                println!("------------------------------------");
            }
        }
        i += 1;
        assert!(i<10000, "Max loop counter exceeded");
    }

    let initialized_graph = executor.push_state.graph_stack.get(0).unwrap();
    let column_ids = test_utils::read_int_vector_parameter(&executor, "COLUMN_IDS".to_string()).unwrap();
    let input_ids = test_utils::read_int_vector_parameter(&executor, "INPUT_IDS".to_string()).unwrap();

    let mut idx = 0;
    for iid in input_ids {
        let input_node_state = initialized_graph.get_state(&(iid as usize)).unwrap(); 
        assert_eq!(test_input[idx] == 1, input_node_state == state_input_on);
        assert_eq!(test_input[idx] == 0, input_node_state == state_input_off); 
        idx += 1;
    }

    println!("Initialized Graph = {}", initialized_graph.to_string());
    // Test overlap update
    // For each column count the connected input nodes
    for cid in column_ids {
        let mut overlap = 0.0;
        let mut boost_factor = 0.0;
        let mut actual_overlap = 0.0;

        // Get Boost cell
        let column_edges = initialized_graph.edges.get(&(cid as usize)).unwrap();
        for ce in column_edges.iter() {
            if initialized_graph.get_state(&ce.get_origin_id()).unwrap() == state_boost_cell {
                let boost_cell_edges = initialized_graph.edges.get(&ce.get_origin_id()).unwrap();
                for bce in boost_cell_edges.iter() {
                    if initialized_graph.get_state(&bce.get_origin_id()).unwrap() == state_input_on {
                        overlap += 1.0;
                    } 
                    boost_factor = initialized_graph.get_weight(&(cid as usize), &ce.get_origin_id()).unwrap();
                    actual_overlap = get_overlap(&initialized_graph, &state_boost_cell, &(cid as usize)).unwrap();
                }
            }
        }
        let expected_overlap = overlap * boost_factor;
        println!("COL = {}, EXP OL = {}, ACT OL = {}", cid, expected_overlap, actual_overlap);
        assert_eq!(expected_overlap, actual_overlap);
    }
}

fn inject_test_graph(executor: &mut PushExecutor, core_id: &mut i32, radius: i32, nodes_within_radius: &mut Vec<usize> ) {
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let sp_code = include_str!("../src/core/spatial_pooler.push").to_string();

    let param_sources = Source::read_debug_code(p_code);
    let sp_sources = Source::read_debug_code(sp_code);

    executor.load(param_sources);
    executor.load(sp_sources);
   
    executor.step_until("BP4".to_string());

    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap();
    let state_column_inactive = test_utils::read_int_parameter(&executor, "STATE_COLUMN_INACTIVE".to_string()).unwrap();
    let state_boost_cell = test_utils::read_int_parameter(&executor, "STATE_BOOST_CELL".to_string()).unwrap();

    // Inject test graph
    let mut test_graph = Graph::new();

    let num_r1_nodes = 3;
    let num_r2_nodes = 2;
    let num_r3_nodes = 3;

    let mut r1_nodes = vec![];
    let mut r2_nodes = vec![];
    let mut r3_nodes = vec![];
    let mut test_col_ids = vec![];

    *core_id = test_graph.add_node(state_column_active) as i32;
    nodes_within_radius.push(*core_id as usize);

    // Radius 1 nodes
    for i in 0..num_r1_nodes {
        let node = test_graph.add_node(state_column_inactive);
        if radius > 0 {
            nodes_within_radius.push(node);
        }
        r1_nodes.push(node);
        test_col_ids.push(node as i32);
        test_graph.add_edge(r1_nodes[i], *core_id as usize, 0.0);
        // Overlap
        let boost_cell = test_graph.add_node(state_boost_cell);
        test_graph.add_edge(boost_cell, r1_nodes[i], i as f32);
    }
    // Add cycle
    assert!(r1_nodes.len()>1);
    test_graph.add_edge(r1_nodes[0], r1_nodes[1], 0.0);

    // Radius 2 nodes
    for j in 0..num_r2_nodes {
        let node = test_graph.add_node(state_column_inactive);
        if radius > 1 {
            nodes_within_radius.push(node);
        }
        r2_nodes.push(node);
        test_col_ids.push(node as i32);
        let r1j = i32::min(j as i32, (r1_nodes.len()-1) as i32) as usize;
        test_graph.add_edge(r2_nodes[j], r1_nodes[r1j], 0.0);
        // Overlap
        let boost_cell = test_graph.add_node(state_boost_cell);
        test_graph.add_edge(boost_cell, r2_nodes[j], (num_r1_nodes+j) as f32);
    }

    // Radius 3 nodes
    for j in 0..num_r3_nodes {
        let node = test_graph.add_node(state_column_active);
        if radius > 2 {
            nodes_within_radius.push(test_graph.add_node(state_column_inactive));
        }
        r3_nodes.push(node);
        test_col_ids.push(node as i32);
        let r2j = i32::min(j as i32, (r2_nodes.len()-1) as i32) as usize;
        test_graph.add_edge(r3_nodes[j], r2_nodes[r2j], 0.0);
        // Overlap
        let boost_cell = test_graph.add_node(state_boost_cell);
        test_graph.add_edge(boost_cell, r3_nodes[j], (num_r1_nodes+num_r2_nodes+j) as f32);
    }

    // Replace default graph with test graph
    executor.push_state.graph_stack.flush();
    executor.push_state.graph_stack.push(test_graph);
    let num_cols = test_col_ids.len() as i32;
    let test_col_ids_item = Item::intvec(IntVector::new(test_col_ids));
    executor.push_state.name_bindings.insert("COLUMN_IDS".to_string(), test_col_ids_item);
    executor.push_state.name_bindings.insert("NUM_COLUMNS".to_string(), Item::int(num_cols));
    executor.push_state.name_bindings.insert("NUM_ACT_COL_PER_INH_AREA".to_string(), Item::int(1));
}

#[test]
fn spatial_pooler_calculate_column_neighborhood() {
    let mut executor = PushExecutor::new();
    let mut nodes_within_radius: Vec<usize> = vec![];
    let mut core_id = 0;
    let test_radius = 2 as i32;
    inject_test_graph(&mut executor, &mut core_id, test_radius, &mut nodes_within_radius);
    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap();
    let state_column_inactive = test_utils::read_int_parameter(&executor, "STATE_COLUMN_INACTIVE".to_string()).unwrap();

    executor.push_state.int_vector_stack.push(IntVector::new(vec![state_column_inactive, state_column_active]));
    executor.push_state.int_vector_stack.push(IntVector::new(vec![]));
    executor.push_state.int_stack.push(test_radius);
    executor.push_state.int_stack.push(core_id);
    executor.push_state.exec_stack.flush();
    executor.push_state.exec_stack.push(Item::id("COLUMN_NEIGHBORHOOD".to_string()));

    let mut instruction_set = InstructionSet::new();
    instruction_set.load();
    let icache = instruction_set.cache();
    let mut i = 0;
    let mut print_on = false;

    println!("GRAPH = {}", executor.push_state.graph_stack.get(0).unwrap().to_string());

    loop {
        if PushInterpreter::step(&mut executor.push_state, &mut instruction_set, &icache) {
          break;
        }
        if let Some(next_instruction) = executor.push_state.exec_stack.get(0) {
            if next_instruction.to_string().starts_with("PRINT") {
                print_on = !print_on;
            }
            if print_on {
                if executor.push_state.exec_stack.size() > 0 {
                    println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                }
                println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                println!("BOOLVECTOR = {}", executor.push_state.bool_vector_stack.to_string());
                println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                println!("------------------------------------");
            }
        }
        i += 1;
        assert!(i<1000, "Max loop counter exceeded");
    }

    assert_eq!(executor.push_state.int_vector_stack.size(), 2);
    let identified_nodes = executor.push_state.int_vector_stack.copy(0).unwrap().values;
    println!("Identified Nodes = {:?}", identified_nodes);
    println!("Actual Nodes = {:?}", nodes_within_radius);
    for nwr in &nodes_within_radius {
        assert!(identified_nodes.contains(&(*nwr as i32)));
    }
    for idn in identified_nodes {
        assert!(nodes_within_radius.contains(&(idn as usize)));
    }

}

#[test]
fn spatial_pooler_calculate_min_local_activity() {
    let mut executor = PushExecutor::new();
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let sp_code = include_str!("../src/core/spatial_pooler.push").to_string();

    let param_sources = Source::read_debug_code(p_code);
    let sp_sources = Source::read_debug_code(sp_code);
                                                        
    executor.load(param_sources);
    executor.load(sp_sources);

    executor.step_until("BP3".to_string());

    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap() as usize;
    let state_column_inactive = test_utils::read_int_parameter(&executor, "STATE_COLUMN_INACTIVE".to_string()).unwrap() as usize;
    let state_boost_cell = test_utils::read_int_parameter(&executor, "STATE_BOOST_CELL".to_string()).unwrap();
    let inhibition_radius = test_utils::read_int_parameter(&executor, "INHIBITION_RADIUS".to_string()).unwrap();
    let num_inputs = test_utils::read_int_parameter(&executor, "NUM_INPUTS".to_string()).unwrap() as usize;
    let column_ids = test_utils::read_int_vector_parameter(&executor, "COLUMN_IDS".to_string()).unwrap();
    let num_active_columns_per_inhibition_area = test_utils::read_int_parameter(&executor, "NUM_ACT_COL_PER_INH_AREA".to_string()).unwrap() as usize;
    let stimulus_threshold = test_utils::read_float_parameter(&executor, "STIMULUS_THRESHOLD".to_string()).unwrap();


    // Inject input vector
    let mut test_input = vec![1; num_inputs];
    for i in 0..num_inputs >> 1 {
        test_input[i] = 0;
    }
    test_input[1] = 0;


    executor
        .push_state
        .input_stack
        .push(PushMessage::new(IntVector::new(vec![]), BoolVector::from_int_array(test_input.clone())));







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
            if next_instruction.to_string().starts_with("BP7") {
                break;
            }
            if print_on {
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
        }
        i += 1;
        assert!(i<20000, "Max loop counter exceeded");
    }

    let test_graph = executor.push_state.graph_stack.pop().unwrap();

    println!("GRAPH = {}", test_graph.to_string());

    for cid in column_ids {

        // Get min local acitvity
        let mut visited_nodes = vec![];
        let mut local_activity = vec![];
        let states = vec![state_column_inactive, state_column_active];
        let distance = inhibition_radius as i32;
        dfs(&test_graph, &mut visited_nodes, &states, cid as usize, distance);
        println!("Neighbors for column {}:{:?} ", cid, visited_nodes);
        for nid in visited_nodes {
            if let Some(ol) = get_overlap(&test_graph, &state_boost_cell, &nid) {
                local_activity.push(ol);
            } else {
                assert!(false, "No overlap could be computed");
            }
        }
        local_activity.sort_by(|a,b| a.partial_cmp(b).unwrap());
        println!("Local activity = {:?} ", local_activity);
        let min_local_activity = local_activity.get(num_active_columns_per_inhibition_area).unwrap();

        // Get overlap
        let mut overlap = 0.0;
        if let Some(ol) = get_overlap(&test_graph, &state_boost_cell, &(cid as usize)) {
            overlap = ol;
        } else {
            assert!(false, "No overlap could be computed");
        }
        let column_is_active = test_graph.get_state(&(cid as usize)).unwrap() == state_column_active as i32;
        println!("Column {} is active: {}", cid, column_is_active);
        println!("OL = {}, MIN = {}, STIM = {}", overlap, min_local_activity, stimulus_threshold);
        assert_eq!(overlap > min_local_activity-0.01 && overlap > stimulus_threshold, column_is_active);
    }

}

fn get_overlap(test_graph: &Graph, state_boost_cell: &i32, node_id: &usize) -> Option<f32> {
    if let Some(incoming_edges) = test_graph.edges.get(node_id) {
        for edge in incoming_edges {
            if let Some(origin_state) = test_graph.get_state(&edge.get_origin_id()) {
                if origin_state == *state_boost_cell {
                    if let Some(edge_weight) = test_graph.get_weight(&edge.get_origin_id(), node_id) {
                        return Some(edge_weight);
                    }
                }
            }
        }
    }
    None
}



fn dfs(graph: &Graph, visited_nodes: &mut Vec<usize>, states: &Vec<usize>, node_id: usize, remaining_distance: i32) {
    if remaining_distance < 0 {
        return;
    }
    if !visited_nodes.contains(&node_id) {
        visited_nodes.push(node_id);
    }
    if let Some(incoming_edges) = graph.edges.get(&node_id) {
        for edge in incoming_edges {
            if let Some(origin_state) = graph.get_state(&edge.get_origin_id()) {
                if states.len() == 0 || states.contains(&(origin_state as usize)) {
                    dfs(graph, visited_nodes, states, edge.get_origin_id(), remaining_distance-1);
                }
            }
        }
    }
}


#[test]
fn spatial_pooler_permanence_adjustment() {
    let mut executor = PushExecutor::new();
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let sp_code = include_str!("../src/core/spatial_pooler.push").to_string();

    let param_sources = Source::read_debug_code(p_code);
    let sp_sources = Source::read_debug_code(sp_code);
                                                         
    executor.load(param_sources);
    executor.load(sp_sources);

    executor.step_until("BP3".to_string());

    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap() as usize;
    let state_boost_cell = test_utils::read_int_parameter(&executor, "STATE_BOOST_CELL".to_string()).unwrap();
    let state_input_on = test_utils::read_int_parameter(&executor, "STATE_INPUT_ON".to_string()).unwrap();
    let state_input_off = test_utils::read_int_parameter(&executor, "STATE_INPUT_OFF".to_string()).unwrap();
    let num_inputs = test_utils::read_int_parameter(&executor, "NUM_INPUTS".to_string()).unwrap() as usize;
    let column_ids = test_utils::read_int_vector_parameter(&executor, "COLUMN_IDS".to_string()).unwrap();
    let permanence_increment = test_utils::read_float_parameter(&executor, "PERM_INCREMENT".to_string()).unwrap();
    let permanence_decrement = test_utils::read_float_parameter(&executor, "PERM_DECREMENT".to_string()).unwrap();

    // Inject input vector
    let mut test_input = vec![1; num_inputs];
    for i in 0..num_inputs >> 1 {
        test_input[i] = 0;
    }
    test_input[1] = 0;

    executor
        .push_state
        .input_stack
        .push(PushMessage::new(IntVector::new(vec![]), BoolVector::from_int_array(test_input.clone())));

    // Execute until end of phase 3
    executor.step_until("BP7".to_string());

    // Extract permanences before Phase 4 execution
    let pre_phase4_graph = executor.push_state.graph_stack.copy(0).unwrap(); 

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
            if next_instruction.to_string().starts_with("BP8") {
                break;
            }
            if print_on {
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
        }
        i += 1;
        assert!(i<20000, "Max loop counter exceeded");
    }

    let post_phase4_graph = executor.push_state.graph_stack.pop().unwrap();
    println!("POST GRAPH = {}", post_phase4_graph.to_string());

    for cid in column_ids {

        // Get min local acitvity
        let column_is_active = pre_phase4_graph.get_state(&(cid as usize)).unwrap() == state_column_active as i32;
        if column_is_active {
            println!("Column = {} ", cid);
            if let Some(incoming_edges) = pre_phase4_graph.edges.get(&(cid as usize)) {
                for edge in incoming_edges {
                    let boost_cell = edge.get_origin_id();
                    if let Some(cell_state) = pre_phase4_graph.get_state(&boost_cell) {
                        if cell_state == state_boost_cell {
                            if let Some(input_edges) = pre_phase4_graph.edges.get(&boost_cell) {
                                for iedge in input_edges {
                                    let input_cell = iedge.get_origin_id();
                                    if let Some(input_state) = pre_phase4_graph.get_state(&input_cell) {
                                        println!("BOOST = {}, INPUT = {}, STATE = {}", boost_cell, input_cell, input_state);
                                         if state_input_on == input_state {

                                            // Expect permanence increase
                                            let pre_phase4_permanence = pre_phase4_graph.get_weight(&input_cell, &boost_cell).unwrap();
                                            let post_phase4_permanence = post_phase4_graph.get_weight(&input_cell, &boost_cell).unwrap();
                                            println!("ON: pre_perm = {}, post_perm = {}", pre_phase4_permanence, post_phase4_permanence);
                                            assert!(f32::abs(pre_phase4_permanence + permanence_increment - post_phase4_permanence) < 0.001);
                                        } else if state_input_off == input_state {
                                              // Expect permanence increase
                                            let pre_phase4_permanence = pre_phase4_graph.get_weight(&input_cell, &boost_cell).unwrap();
                                            let post_phase4_permanence = post_phase4_graph.get_weight(&input_cell, &boost_cell).unwrap();
                                            println!("OFF: pre_perm = {}, post_perm = {}", pre_phase4_permanence, post_phase4_permanence);
                                            assert!(f32::abs(pre_phase4_permanence - permanence_decrement - post_phase4_permanence) < 0.001);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

}

/// Injects a stack of graphs with a star topology to test the activation and overlap duty
/// cycles of the center column.
pub fn inject_duty_cycle_test_graphs(executor: &mut PushExecutor, center_col_id: &usize, center_boost_cell_id: &mut usize, expected_boost_factor: &mut f32) {

    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap();
    let state_column_inactive = test_utils::read_int_parameter(&executor, "STATE_COLUMN_INACTIVE".to_string()).unwrap();
    let state_boost_cell = test_utils::read_int_parameter(&executor, "STATE_BOOST_CELL".to_string()).unwrap();
    let topology_weight = test_utils::read_float_parameter(executor, "TOPOLOGY_WEIGHT".to_string()).unwrap();
    let window_length = test_utils::read_int_parameter(executor, "WIN_LENGTH".to_string()).unwrap();
    let boosting_strength = test_utils::read_float_parameter(executor, "BOOSTING_STRENGTH".to_string()).unwrap();


    // Inject test graph
    let num_neigbors = 5;
    let mut test_graph = Graph::new();
    let mut test_col_ids = vec![0;num_neigbors];
    let mut test_boost_cell_ids = vec![0;num_neigbors];


    let test_center_activity = 0.2;
    let test_neighbor_activity = 0.3;

    let test_center_state = state_column_active;
    let test_neighbor_state = state_column_active;

    // Center column
     // Add column
    let center_col_id = test_graph.add_node(state_column_active);
    // Add actvity duty cycle
    test_graph.add_edge(center_col_id, center_col_id, test_center_activity);
    // Add boost cell ID
    let center_boost_cell_id = test_graph.add_node(state_boost_cell);
    // Add overlap edge 
    test_graph.add_edge(center_boost_cell_id, center_col_id, 1.0);
    // Add boost edge
    test_graph.add_edge(center_col_id, center_boost_cell_id, 1.0);

    // Neighbors in star formation
    let mut nb_idx = 0;
    while nb_idx < num_neigbors {
        // Add column
        test_col_ids[nb_idx] = test_graph.add_node(state_column_inactive);
        // Add actvity duty cycle
        test_graph.add_edge(test_col_ids[nb_idx], test_col_ids[nb_idx], test_neighbor_activity);
        // Add connection to center
        test_graph.add_edge(test_col_ids[nb_idx], center_col_id, topology_weight);
        // Add boost cell ID
        test_boost_cell_ids[nb_idx] = test_graph.add_node(state_boost_cell);
        // Add overlap edge 
        test_graph.add_edge(test_boost_cell_ids[nb_idx], test_col_ids[nb_idx], 1.0);
        // Add boost edge
        test_graph.add_edge(test_col_ids[nb_idx], test_boost_cell_ids[nb_idx], 1.0);
        nb_idx = nb_idx +1;
    }

    let mut next_step_test_graph = test_graph.clone();
    executor.push_state.graph_stack.push(test_graph);

    // Reset activity duty cycle
    next_step_test_graph.set_weight(&center_col_id, &center_col_id, 0.0);
    next_step_test_graph.set_state(&center_col_id, test_center_state);

    nb_idx = 0;
    while nb_idx < num_neigbors {
        next_step_test_graph.set_weight(&test_col_ids[nb_idx], &test_col_ids[nb_idx], 0.0);
        next_step_test_graph.set_state(&test_col_ids[nb_idx], test_neighbor_state);
        nb_idx += 1;
    }

   // TODO: calculate boost factor for center column 
   let neighbors_active = if test_neighbor_state == state_column_active { 1.0 } else { 0.0 };
   let neighbor_duty_cycle = ((window_length-1) as f32* test_neighbor_activity + neighbors_active) / (window_length as f32);
        
   let center_duty_cycle = ((window_length-1) as f32* test_center_activity + neighbors_active) / (window_length as f32);
        
   *expected_boost_factor = f32::exp(-1.0*boosting_strength * (center_duty_cycle - neighbor_duty_cycle));

    executor.push_state.graph_stack.push(next_step_test_graph);

    let mut all_columns = vec![];
    all_columns.push(center_col_id as i32);
    for tc in test_col_ids {
        all_columns.push(tc as i32);
    }
    executor.push_state.name_bindings.insert("COLUMN_IDS".to_string(), Item::intvec(IntVector::new(all_columns)));

}


//#[test]
fn spatial_pooler_duty_cycle() {
    let mut executor = PushExecutor::new();
    executor.initialize();

    let p_code = include_str!("../src/core/parameters.push").to_string();
    let sp_code = include_str!("../src/core/spatial_pooler.push").to_string();

    let param_sources = Source::read_debug_code(p_code);
    let sp_sources = Source::read_debug_code(sp_code);
                                                         
    executor.load(param_sources);
    executor.load(sp_sources);

    executor.step_until("BP3".to_string());

    let state_column_active = test_utils::read_int_parameter(&executor, "STATE_COLUMN_ACTIVE".to_string()).unwrap() as usize;
    let num_inputs = test_utils::read_int_parameter(&executor, "NUM_INPUTS".to_string()).unwrap() as usize;
    let state_boost_cell = test_utils::read_int_parameter(&executor, "STATE_BOOST_CELL".to_string()).unwrap();
    let state_input_on = test_utils::read_int_parameter(&executor, "STATE_INPUT_ON".to_string()).unwrap();

    // Inject input vector
    let mut test_input = vec![1; num_inputs];
    for i in 0..num_inputs >> 1 {
        test_input[i] = 0;
    }
    test_input[1] = 0;

    executor
        .push_state
        .input_stack
        .push(PushMessage::new(IntVector::new(vec![]), BoolVector::from_int_array(test_input.clone())));

    // Execute until end of phase 3
    executor.step_until("BP8".to_string());
     //assert!(false);
    // Eject exiting graphs
    executor.push_state.graph_stack.flush();

    // Inject test graphs
    let mut expected_boost_factor = 0.0;
    let mut test_id = 0;
    let mut test_boost_cell = 0;
    inject_duty_cycle_test_graphs(&mut executor, &mut test_id, &mut test_boost_cell, &mut expected_boost_factor);


    println!("GRAPH STACK = {}", executor.push_state.graph_stack.to_string());
    let test_columns = test_utils::read_int_vector_parameter(&executor, "COLUMN_IDS".to_string()).unwrap();
    println!("Column IDs = {:?}", test_columns);
    println!("Graph =  {}", executor.push_state.graph_stack.get(0).unwrap().to_string());
    
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
            if next_instruction.to_string().starts_with("BP10") {
                break;
            }
            if next_instruction.to_string().starts_with("PRINTGRAPH") {
                println!("GRAPH = {}", executor.push_state.graph_stack.to_string());
            }
            if print_on {
                if executor.push_state.exec_stack.size() > 0 {
                    println!("EXEC = {}", executor.push_state.exec_stack.copy(0).unwrap().to_string());
                }
                println!("INTEGER = {}", executor.push_state.int_stack.to_string());
                println!("INTVECTOR = {}", executor.push_state.int_vector_stack.to_string());
                println!("FLOATVECTOR = {}", executor.push_state.float_vector_stack.to_string());
                println!("BOOL = {}", executor.push_state.bool_stack.to_string());
                println!("FLOAT = {}", executor.push_state.float_stack.to_string());
                println!("INDEX = {}", executor.push_state.index_stack.to_string());
                println!("------------------------------------");
            }
        }
        i += 1;
        assert!(i<20000, "Max loop counter exceeded");
    }

    let test_graph = executor.push_state.graph_stack.pop().unwrap();
    let actual_boost_factor = test_graph.get_weight(&test_id, &test_boost_cell).unwrap();
    println!("Exp. BF = {}, Actual BF = {}", expected_boost_factor, actual_boost_factor);
    assert_eq!(expected_boost_factor, actual_boost_factor);
}

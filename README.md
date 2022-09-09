## Push Hierarchical Temporal Memory (PHTM)

[![example workflow](https://github.com/johker/phtm/blob/master/.github/workflows/rust.yml/badge.svg)](https://github.com/johker/phtm/blob/master/.github/workflows/rust.yml)
[![example workflow](https://github.com/johker/phtm/blob/master/.github/workflows/node.js.yml/badge.svg)](https://github.com/npm/johker/phtm/blob/master/.github/workflows/node.js.yml)

Implementation of Numenta's [Hierarchical Temporal Memory (HTM) algorithm](https://numenta.com/resources/biological-and-machine-intelligence/) in the 
Push language. Push is a stack-based, Turing-complete programming language that enables autoconstructive evolution in its programs.
More information can be found [here](http://faculty.hampshire.edu/lspector/push.html). The [Pushr](https://github.com/johker/pushr) virtual machine is used to run the spatial pooler and the temporal memory. 


### Details

- core contains the implementation of the spatial pooler and the temporal memory
- proxy contains the message broker
- msg contains the encoding of data in the message payload
- ui contains the d3 visualization

### How to run

Start the spatial pooler with
```  cargo run "$(< src/core/spatial_pooler.push)" ```

Start the temporal memory with
```  cargo run "$(< src/core/temporal_memory.push)" ```


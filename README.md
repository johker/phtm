## Push Hierarchical Temporal Memory (PHTM)

![Tests](https://github.com/johker/phtm/actions/workflows/rust.yml/badge.svg)

Implementation of Numenta's [Hierarchical Temporal Memory (HTM) algorithm](https://numenta.com/resources/biological-and-machine-intelligence/) in the 
Push language. Push is a stack-based, Turing-complete programming language that enables autoconstructive evolution in its programs.
More information can be found [here](http://faculty.hampshire.edu/lspector/push.html). The [Pushr](https://github.com/johker/pushr) virtual machine is used to run the spatial pooler and the temporal memory. 


### Details

- core contains the implementation of the spatial pooler and the temporal memory
- proxy contains the message broker
- msg contains the encoding of data in the message payload
- ui contains the d3 visualization


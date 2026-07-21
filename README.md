# grust_pcg
## About
Grust PCG is a plugin for godot that uses a node graph to procedurally spawn/instance meshes in your godot levels
## Installation
### Requirements
- Cargo

Idk how do releases yet so you will need to build from source. 
Clone this folder into your addons folder in your godot project: res://addons
Build both the pcg/graph_editor and pcg/godot_pcg, open both folders in your terminal and with cargo installed, run cargo build
Due to the current method of locationg the graph editor executable, you must put the plugin directly in the addons folder in your project, if you can't open the graph editor this is probably the problem

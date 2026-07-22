# grust_pcg
<img width="1710" height="1107" alt="image" src="https://github.com/user-attachments/assets/bd014e15-2357-4c51-848a-74f3586b00fb" />
Sample Graph:
<img width="3420" height="945" alt="image" src="https://github.com/user-attachments/assets/2ac2d291-bc47-424b-a03d-f692e2e8b0a3" />

### Note
This project is in a very very early stage of development. Many things will change. Some things will not work. I'm also building this on a MacBook Air and havent tested the plugin on Windows or Linux yet (I use CachyOS btw, I will test on linux and windows soon). It might work, it might not. It also has limited fuctionality at the moment but you can add to if if you want. Many new features are in the works. 

**Scroll down for instalation structions**

## About
Grust PCG is a plugin for godot that uses a node graph to procedurally spawn/instance meshes in your godot levels. PCG (Procedural Content Generation) is a modern workflow that allows developers and artists to create worlds/scenes with math, rather than handplaceing every single object. A good PCG framework allows artists/developers to maintain an artistic control over their world while significantly decreasing the amount of time required. Most people don't want to place 1 billion blades of grass by hand.

This plugin is split into 3 main parts:
- The Graph Editor
- The Godot Plugin
- A shared library that contains code used by both

Everything is made in Rust. Why? Well there are a few advantages:
- Decoupled from Godot
  - While Godot and GDScript are amazing tools which I heavily support and use to make games of my own, Rust provides more independance and oppertunity for future endevors...
- Rust is very modern, and BLAZINGLY FAST (thats right 😁). Some expensive opperations may preform better on Rust than GDScript.
- Rust forces you to write good code, GDScript does this to a lesser extent. Because of this, Rust projects tend to be more stable and sometimes even easier to maintain.
- Rust has a large library of Crates (packages) that make some things easier.
- Rust is one of the most hyped programming languages right now so not only did I want to learn it (by making a project) but many other people might be on the same train as me. I have been using this language for a while now and I really enjoy programming in it compared to GDScript. No regrets.

The Graph Editor is also not made in Godot, it uses a Rust library called Iced. Why? Well there are a few advantages:
- Decoupled from Godot
  - Because the whole thing is made in Rust, the less it has to talk to Godot the better. While you can program in Godot in any language, that doesn't mean it's easy. Being able to use a native Rust GUI library is a breath of fresh air.

Some day I believe we could have an open standard for PCG graphs, where you can choose any open-sourced graph editor and make a PCG graph, and then those graphs could run in any application, from Unreal Engine, to Godot, to Houdini, to Blender. If you think about it, we already have open standards for things like shaders (GLSL) and meshes (gltf/glb). Why not have something for PCG aswell? Having a seperate graph editor is a stepping stone too this goal.

## How it works
Currently the way this functions is you open the graph editor from Godot (Project -> Tools -> Open Graph Editor, or by clicking the button on the PCG Zone object), if you pin it to your taskbar that should work too. The editor can load, edit, and save .pcg files which can be loaded into a PCG Zone and run in Godot. The PCG Zone is where you set input for your graph, like Splines (Path3Ds) and Meshes.

## Installation
### Requirements
- Rust (you really just need cargo but it's probably better to just install the whole thing)
  - Go to [rustup](rustup.rs) -This is the best way to install it, if you need help google "How to install Rust on -your opperating system-"

Currently, the only way to use this is to build it from source.
- Clone this folder into your addons folder in your godot project: res://addons
- Build both the pcg/graph_editor and pcg/godot_pcg, open each folder in your terminal and with cargo installed, run ```cargo build```
  - To open a folder in terminal, I would right click it in your file explorer/manager and there may be an option to "open in terminal".
  - You can also just open the terminal and type ```cd --insert folder path here--```. cd is Bash for change directory.
  - If cargo actually does something and doesn't spit out an error that means you did it right.
- Due to the current method of locationg the graph editor executable, you must put the plugin directly in the addons folder in your project, if you can't open the graph editor this is probably the problem.
- Your godot project folder should look something like /project_name/addons/grust_pcg/...

## Planed Features
The biggest goal right now is getting to a point where this tool is quick, intuative and powerful.
### Graph Complexity
- Many new nodes
- Custom point attributes
- Distance based nodes
- Custom Nodes???
- Preformance Improvements
### Artistic Control
- Better input and variable system
- Spawning with normals (slant)
- Scene spawning
- Sockets (add points to a mesh that can be used to procedurally spawn other meshes on)
- Maybe some day we can support runtime graphs...
### Ease of use
- Better UI
- Smoother and more intuitive editing
- Installer/Integration with Godot plugin ecosystem.
- Documentation and tutorials

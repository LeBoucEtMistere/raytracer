# Raytracer

A Rust implementation of a basic raytracer based on Peter Shirley's blog ["Ray Tracing in One Weekend"](https://raytracing.github.io/books/RayTracingInOneWeekend.html)

This project is mostly for me to play with Rust, raytracing, and performances. It is implemented in a lib, but comes with examples scenes in the examples folder.

I try to keep render times fast by using multi-threaded code and performant datastructures (BVH), and by doing optimization (usage of flamegraphs and benchmarking).

Example image generated:

![Rendering spheres](https://github.com/LeBoucEtMistere/raytracer/blob/master/renders/render1.png?raw=true)

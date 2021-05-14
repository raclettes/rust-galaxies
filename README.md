# Galaxies
Simple galaxy simulation in Rust

## Features
- Attempts to handle collisions by checking their relative velocities
- Traces future paths *by simulating seperately* (as shown below)

  ![Path tracing](https://i.discord.fr/Aj6.png)
  ![Zoomed in](https://i.discord.fr/HBh.png)

## Config
The default configuration file looks like:
```yaml
galaxies:
  - star:
      mass: 2000000.0
      color: [1.0, 1.0, 0.0, 1.0]
    planets:
      distance:
        min: 10
        max: 30
      mass:
        min: 1
        max: 50
      number: 250
    position: [60.0, 60.0]
    direction: 1.0
  - star:
      mass: 1000000.0
      color: [1.0, 1.0, 0.0, 1.0]
    planets:
      distance:
        min: 10
        max: 30
      mass:
        min: 1
        max: 50
      number: 250
    position: [-60.0, -60.0]
    direction: -1.0
scale: 1.0
time_scale: 1.0
```

This has 3 components:
- `galaxies`, which refers to the structures that will be generated. Each contains information about the central star, planets' distances and masses, the position in the world and the direction of the planets' orbits. Planets will then be generated randomly about the star with the given velocity `v = sqrt((M_star+m_planet)/distance)`.
- `scale` refers to the screen:real ratio when drawing. Bigger numbers are more zoomed in, smaller numbers are more zoomed out.<sup>1</sup>
- `time_scale` refers to the passing of time. This, alongside the frame rate, affects the simulation's speed. All forces and velocities are multiplied by `dt * time_scale`<sup>2</sup>

**1**: Use <kbd>Z</kbd> to zoom out, <kbd>X</kbd> to zoom in

**2**: Use <kbd>A</kbd> to slow down time, <kbd>S</kbd> to speed up time.

You can also use the arrow keys to move around.

## Running
As always, use

```bash
cargo run
```

To build and run.
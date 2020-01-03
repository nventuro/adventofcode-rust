use core::ops::{Add, AddAssign};
use std::collections::HashSet;
use std::fs;

extern crate regex;
use regex::Regex;

extern crate num_integer;

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd)]
struct Coordinate(i32);

impl Coordinate {
    fn new(value: i32) -> Coordinate {
        Coordinate { 0: value }
    }

    fn energy(&self) -> u32 {
        self.0.abs() as u32
    }
}

impl Add for Coordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Coordinate::new(self.0 + rhs.0)
    }
}

impl AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Particle {
    position: Coordinate,
    velocity: Coordinate,
}

impl Particle {
    fn new(position: i32) -> Particle {
        Particle {
            position: Coordinate::new(position),
            velocity: Coordinate::new(0),
        }
    }

    fn gravitate(&mut self, other: &Particle) {
        if self.position < other.position {
            self.velocity.0 += 1;
        } else if other.position < self.position {
            self.velocity.0 -= 1;
        }
    }

    fn step(&mut self) {
        self.position += self.velocity;
    }

    fn potential_energy(&self) -> u32 {
        self.position.energy()
    }

    fn kinetic_energy(&self) -> u32 {
        self.velocity.energy()
    }
}

const DIMENSIONS: usize = 3;

#[derive(Clone, Eq, Hash, PartialEq)]
struct UnidimensionalSystem {
    particles: Vec<Particle>,
}

fn extract<T>(elements: &mut [T], index: usize) -> (&mut T, impl Iterator<Item = &T>) {
    let (before, remainder) = elements.split_at_mut(index);
    let (extracted, after) = remainder.split_at_mut(1);

    (&mut extracted[0], before.iter().chain(after.iter()))
}

impl UnidimensionalSystem {
    fn new(particles: Vec<Particle>) -> UnidimensionalSystem {
        UnidimensionalSystem { particles }
    }

    fn step(&mut self) {
        for i in 0..self.particles.len() {
            let (particle, others) = extract(&mut self.particles, i);
            for other in others {
                particle.gravitate(other);
            }
        }

        for particle in &mut self.particles {
            particle.step();
        }
    }
}

struct System {
    axes: [UnidimensionalSystem; DIMENSIONS],
    n_bodies: usize,
}

impl System {
    fn new(bodies: Vec<[Particle; DIMENSIONS]>) -> System {
        let mut axes = (0..DIMENSIONS)
            .map(|dimension| {
                bodies
                    .iter()
                    .map(|body| body[dimension])
                    .collect::<Vec<_>>()
            })
            .map(|particles| UnidimensionalSystem::new(particles))
            .collect::<Vec<_>>();

        assert_eq!(axes.len(), DIMENSIONS);

        System {
            axes: [axes.remove(0), axes.remove(0), axes.remove(0)],
            n_bodies: bodies.len(),
        }
    }

    fn step(&mut self) {
        for subsystem in &mut self.axes {
            subsystem.step();
        }
    }

    fn total_energy(&self) -> u32 {
        (0..self.n_bodies)
            .map(|body| {
                let kinetic = (0..DIMENSIONS)
                    .map(|dimension| self.axes[dimension].particles[body].kinetic_energy())
                    .sum::<u32>();

                let potential = (0..DIMENSIONS)
                    .map(|dimension| self.axes[dimension].particles[body].potential_energy())
                    .sum::<u32>();

                kinetic * potential
            })
            .sum()
    }

    fn find_period(&self) -> usize {
        let mut periods = Vec::new();

        for dimension in 0..DIMENSIONS {
            let mut subsystem = self.axes[dimension].clone();

            let mut old_states = HashSet::<UnidimensionalSystem>::new();
            loop {
                subsystem.step();
                if !old_states.insert(subsystem.clone()) {
                    break;
                }
            }
            periods.push(old_states.len());
        }

        periods
            .iter()
            .fold(1, |accum, period| num_integer::lcm(accum, *period))
    }
}

fn main() {
    let filename = "input";
    let contents = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Failed to read from file '{}'", filename));

    let system = System::new(
        Regex::new(r"<x=(?P<x>-?\d+), y=(?P<y>-?\d+), z=(?P<z>-?\d+)>")
            .unwrap()
            .captures_iter(&contents)
            .map(|capture| {
                [
                    Particle::new(capture["x"].parse().unwrap()),
                    Particle::new(capture["y"].parse().unwrap()),
                    Particle::new(capture["z"].parse().unwrap()),
                ]
            })
            .collect(),
    );

    println!("Total period: {}", system.find_period());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy() {
        let mut system = System::new(vec![
            [Particle::new(-1), Particle::new(0), Particle::new(2)],
            [Particle::new(2), Particle::new(-10), Particle::new(-7)],
            [Particle::new(4), Particle::new(-8), Particle::new(8)],
            [Particle::new(3), Particle::new(5), Particle::new(-1)],
        ]);

        for _ in 0..10 {
            system.step();
        }

        assert_eq!(system.total_energy(), 179);
    }

    #[test]
    fn test_period() {
        let system = System::new(vec![
            [Particle::new(-8), Particle::new(-10), Particle::new(0)],
            [Particle::new(5), Particle::new(5), Particle::new(10)],
            [Particle::new(2), Particle::new(-7), Particle::new(3)],
            [Particle::new(9), Particle::new(-8), Particle::new(-3)],
        ]);

        assert_eq!(system.find_period(), 4686774924);
    }
}

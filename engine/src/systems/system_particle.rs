use rltk::{FontCharType, Point, RGBA};
use shipyard::{
    AllStoragesViewMut, EntityId, Get, IntoIter, IntoWithId, Unique, UniqueView, UniqueViewMut, View, ViewMut,
};

use crate::{
    components::{Lifetime, Particle, Position, Renderable, Velocity},
    effects::{add_effect, EffectType},
    uniques::FrameTime,
    RenderOrder,
};

#[derive(Debug)]
struct ParticleRequest {
    x: i32,
    y: i32,
    vel_x: f32,
    vel_y: f32,
    fg: RGBA,
    bg: RGBA,
    glyph: FontCharType,
    lifetime_ms: f32,
}

#[derive(Debug, Unique)]
pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder { requests: Vec::new() }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        vel_x: f32,
        vel_y: f32,
        fg: RGBA,
        bg: RGBA,
        glyph: FontCharType,
        lifetime_ms: f32,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            vel_x,
            vel_y,
            fg,
            bg,
            glyph,
            lifetime_ms,
        });
    }

    pub fn clear(&mut self) {
        self.requests.clear();
    }
}

pub fn spawn_particles(mut store: AllStoragesViewMut) {
    let mut to_add = vec![];
    {
        let mut particle_builder = store.borrow::<UniqueViewMut<ParticleBuilder>>().unwrap();
        for p in particle_builder.requests.iter() {
            to_add.push((
                Renderable {
                    glyph: p.glyph,
                    fg: p.fg,
                    bg: p.bg,
                    always_render: true,
                    order: RenderOrder::Particle,
                    ..Default::default()
                },
                Position {
                    ps: vec![Point { x: p.x, y: p.y }],
                },
                Velocity { x: p.vel_x, y: p.vel_y },
                Lifetime { ms: p.lifetime_ms },
                Particle {
                    float_x: p.x as f32,
                    float_y: p.y as f32,
                },
            ));
        }
        particle_builder.clear();
    }

    for c in to_add {
        store.add_entity(c);
    }
}

pub fn update_particles(
    frametime: UniqueView<FrameTime>,
    mut vpart: ViewMut<Particle>,
    mut vlifetime: ViewMut<Lifetime>,
    vvel: View<Velocity>,
    mut vpos: ViewMut<Position>,
) {
    for (id, (particle, lifetime)) in (&mut vpart, &mut vlifetime).iter().with_id() {
        lifetime.ms -= frametime.0;

        let vel = vvel.get(id);
        if let Ok(vel) = vel {
            if let Ok(pos) = (&mut vpos).get(id) {
                for pos in pos.ps.iter_mut() {
                    particle.float_x += (vel.x) * (frametime.0 / 1000.0);
                    particle.float_y += (vel.y) * (frametime.0 / 1000.0);
                    pos.x = particle.float_x as i32;
                    pos.y = particle.float_y as i32;
                }
            }
        }
    }

    remove_dead_particles(vlifetime);
}

pub fn remove_dead_particles(vlifetime: ViewMut<Lifetime>) {
    let mut particles_to_remove: Vec<EntityId> = Vec::new();
    for (id, lifetime) in vlifetime.iter().with_id() {
        if lifetime.ms <= 0.0 {
            particles_to_remove.push(id);
        }
    }

    for id in particles_to_remove {
        add_effect(None, EffectType::Delete { entity: id });
    }
}

use crate::components::*;
use crate::*;
use quicksilver::prelude::*;
use std::f32::consts::*;
use std::hash::Hash;
use std::marker::PhantomData;

pub(crate) trait SystemInterface {
    type Update;
    type Refer;
}
pub(crate) trait SystemProcess: SystemInterface {
    fn process(update: &mut Self::Update, _ref: &Self::Refer);
}

pub(crate) struct System<U, R> {
    phantom: PhantomData<(U, R)>,
}
impl<U, R> SystemInterface for System<U, R> {
    type Update = U;
    type Refer = R;
}

pub(crate) struct ForObserverSet();
pub(crate) struct ForObserverCheck();

impl<V, C> SystemProcess
    for System<CContainer<ValueObserver<V, C>>, (&CContainer<C>, ForObserverSet)>
where
    V: PartialEq + Copy,
{
    fn process(observers: &mut Self::Update, (components, _): &Self::Refer) {
        observers
            .iter_mut()
            .zip_entity(components)
            .for_each(|(observer, component)| {
                observer.set(component);
            });
    }
}

impl<V, C> SystemProcess
    for System<CContainer<ValueObserver<V, C>>, (&CContainer<C>, ForObserverCheck)>
where
    V: PartialEq + Copy,
{
    fn process(observers: &mut Self::Update, components: &Self::Refer) {
        observers
            .iter_mut()
            .zip_entity(components.0)
            .for_each(|(observer, component)| {
                observer.check(component);
            });
    }
}

impl SystemProcess
    for System<
        CContainer<CharacterState>,
        (&CContainer<Input>, &CContainer<CharacterAnimEndObserver>),
    >
{
    fn process(states: &mut Self::Update, (inputs, anim_observers): &Self::Refer) {
        states
            .iter_mut()
            .zip_entity2(inputs, anim_observers)
            .for_each(|(state, input, anim_observer)| match state {
                CharacterState::Wait => {
                    if input.attack {
                        *state = CharacterState::Attack;
                    }
                }
                CharacterState::Attack => {
                    if anim_observer.is_changed() {
                        *state = CharacterState::Wait;
                    }
                }
            });
    }
}

impl SystemProcess for System<CContainer<Velocity>, CContainer<Input>> {
    fn process(velocities: &mut Self::Update, inputs: &Self::Refer) {
        velocities
            .iter_mut()
            .zip_entity(inputs)
            .for_each(|(velocity, input)| {
                velocity.x = 0f32;
                velocity.y = 0f32;
                if input.left {
                    velocity.x = -2f32;
                }
                if input.right {
                    velocity.x = 2f32;
                }
                if input.up {
                    velocity.y = -2f32;
                }
                if input.down {
                    velocity.y = 2f32;
                }
            });
    }
}

impl SystemProcess for System<CContainer<MoveTarget>, (&CContainer<Team>, &CContainer<Position>)> {
    fn process(move_targets: &mut Self::Update, team_pos: &Self::Refer) {
        let (teams, positions) = team_pos;
        move_targets
            .iter_mut()
            .zip_entity2(teams, positions)
            .for_each(|(target, self_team, self_pos)| {
                teams
                    .iter()
                    .filter(|(_, team)| team.team_id() != self_team.team_id())
                    .for_each(|(entity_id, _)| {
                        if let Some(pos) = CContainer::<Position>::get(positions, entity_id) {
                            let distance = pos.distance((self_pos.x, self_pos.y));
                            if distance < 100f32 {
                                target.x = pos.x;
                                target.y = pos.y;
                            } else {
                                target.x = self_pos.x;
                                target.y = self_pos.y;
                            }
                        }
                    });
            });
    }
}

impl SystemProcess
    for System<CContainer<Velocity>, (&CContainer<Position>, &CContainer<MoveTarget>)>
{
    fn process(velocities: &mut Self::Update, pos_tgt: &Self::Refer) {
        velocities
            .iter_mut()
            .zip_entity2(pos_tgt.0, pos_tgt.1)
            .for_each(|(vel, pos, target)| {
                let mut tmp = Vector::default();
                tmp.x = target.x - pos.x;
                tmp.y = target.y - pos.y;
                vel.x = tmp.x / 50f32;
                vel.y = tmp.y / 50f32;
            });
    }
}

impl SystemProcess for System<CContainer<Position>, CContainer<Velocity>> {
    fn process(positions: &mut Self::Update, velocities: &Self::Refer) {
        positions
            .iter_mut()
            .zip_entity(velocities)
            .for_each(|(pos, vel)| {
                pos.x += vel.x;
                pos.y += vel.y;
            });
    }
}

impl SystemProcess for System<CContainer<CharacterAnimator>, CContainer<CharacterStateObserver>> {
    fn process(animators: &mut Self::Update, state_observers: &Self::Refer) {
        animators
            .iter_mut()
            .zip_entity(state_observers)
            .for_each(|(anim, state_observer)| {
                if state_observer.is_changed() {
                    match state_observer.value() {
                        CharacterState::Wait => {
                            anim.play(CharacterAnimID::Wait);
                        }
                        CharacterState::Attack => {
                            log::info!("attack");
                            anim.play(CharacterAnimID::Attack);
                        }
                    }
                }
            });
    }
}

impl<K, V> SystemProcess for System<CContainer<Animator<K, V>>, ()>
where
    K: Hash + Eq + Copy,
{
    fn process(animators: &mut Self::Update, _: &Self::Refer) {
        animators.iter_mut().for_each(|(_, a)| a.update());
    }
}

impl SystemProcess for System<CContainer<CharacterView>, CContainer<CharacterAnimator>> {
    fn process(views: &mut Self::Update, animators: &Self::Refer) {
        views
            .iter_mut()
            .zip_entity(animators)
            .for_each(|(view, animator)| {
                if let Some(val) = animator.value() {
                    view.radius_scale = val.radius_scale;
                    view.weapon_direction = val.weapon_direction;
                }
            });
    }
}

impl SystemProcess
    for System<CContainer<CharacterView>, (&CContainer<Position>, &CContainer<Velocity>)>
{
    fn process(views: &mut Self::Update, pos_vel: &Self::Refer) {
        views
            .iter_mut()
            .zip_entity2(pos_vel.0, pos_vel.1)
            .for_each(|(view, pos, vel)| {
                view.position.x = pos.x;
                view.position.y = pos.y;
                if vel.x != 0f32 || vel.y != 0f32 {
                    view.direction = vel.y.atan2(vel.x);
                }
            });
    }
}

impl SystemProcess for System<Window, CContainer<CharacterView>> {
    fn process(window: &mut Self::Update, views: &Self::Refer) {
        views.iter().for_each(|(_, view)| {
            // log::info!("r {}", view.radius_scale);

            window.draw(
                &Circle::new(
                    (view.position.x, view.position.y),
                    view.radius * view.radius_scale,
                ),
                Col(view.color),
            );
            let dir = view.direction + view.weapon_direction;
            let line_end = (
                view.position.x + dir.cos() * view.radius * 1.8f32,
                view.position.y + dir.sin() * view.radius * 1.8f32,
            );
            window.draw(
                &Line::new((view.position.x, view.position.y), line_end),
                Col(view.color),
            );
        });
    }
}

use crate::components::*;
use crate::*;
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
            .for_each(|(_, observer, component)| {
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
            .for_each(|(_, observer, component)| {
                observer.check(component);
            });
    }
}

impl SystemProcess for System<CContainer<Velocity>, CContainer<Input>> {
    fn process(velocities: &mut Self::Update, inputs: &Self::Refer) {
        velocities
            .iter_mut()
            .zip_entity(inputs)
            .for_each(|(_, velocity, input)| {
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
    fn process(move_targets: &mut Self::Update, (teams, positions): &Self::Refer) {
        move_targets
            .iter_mut()
            .zip_entity2(teams, positions)
            .for_each(|(_, target, self_team, self_pos)| {
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
            .for_each(|(_, vel, pos, target)| {
                let mut tmp = Vector::default();
                tmp.x = target.x - pos.x;
                tmp.y = target.y - pos.y;
                vel.x = tmp.x / 50f32;
                vel.y = tmp.y / 50f32;
            });
    }
}

impl SystemProcess
    for System<CContainer<Direction>, (&CContainer<Position>, &CContainer<MoveTarget>)>
{
    fn process(directions: &mut Self::Update, (positions, targets): &Self::Refer) {
        directions
            .iter_mut()
            .zip_entity2(positions, targets)
            .for_each(|(_, dir, position, target)| {
                if position != target {
                    *dir = (target.y - position.y).atan2(target.x - position.x);
                }
            });
    }
}

impl SystemProcess
    for System<CContainer<Velocity>, (&CContainer<CharacterView>, &CContainer<CharacterAnimator>)>
{
    fn process(velocities: &mut Self::Update, (views, animators): &Self::Refer) {
        velocities
            .iter_mut()
            .zip_entity2(views, animators)
            .for_each(|(_, velocity, view, animator)| {
                if let Some(val) = animator.value() {
                    if val.move_forward != 0f32 {
                        velocity.x = view.direction.cos() * val.move_forward;
                        velocity.y = view.direction.sin() * val.move_forward;
                    }
                }
            });
    }
}

impl SystemProcess for System<CContainer<Position>, CContainer<Velocity>> {
    fn process(positions: &mut Self::Update, velocities: &Self::Refer) {
        positions
            .iter_mut()
            .zip_entity(velocities)
            .for_each(|(_, pos, vel)| {
                pos.x += vel.x;
                pos.y += vel.y;
            });
    }
}

impl SystemProcess for System<CContainer<Direction>, CContainer<Input>> {
    fn process(directions: &mut Self::Update, inputs: &Self::Refer) {
        directions
            .iter_mut()
            .zip_entity(inputs)
            .for_each(|(_, direction, input)| {
                if input.left {
                    *direction = PI;
                    if input.up {
                        *direction = FRAC_PI_4 * 5f32;
                    }
                    if input.down {
                        *direction = FRAC_PI_4 * 3f32;
                    }
                } else if input.right {
                    *direction = 0f32;
                    if input.up {
                        *direction = FRAC_PI_4 * 7f32;
                    }
                    if input.down {
                        *direction = FRAC_PI_4;
                    }
                } else {
                    if input.up {
                        *direction = FRAC_PI_2 * 3f32;
                    }
                    if input.down {
                        *direction = FRAC_PI_2;
                    }
                }
            });
    }
}

impl SystemProcess
    for System<
        CContainer<SwordCollider>,
        (&CContainer<CharacterView>, &CContainer<CharacterAnimator>),
    >
{
    fn process(sword_colliders: &mut Self::Update, (views, animators): &Self::Refer) {
        sword_colliders
            .iter_mut()
            .zip_entity2(views, animators)
            .for_each(|(_, collider, view, animator)| {
                let dir = view.direction + view.weapon_direction;
                collider.line.a = view.position;
                collider.line.b.x = view.position.x + dir.cos() * view.radius * 1.8f32;
                collider.line.b.y = view.position.y + dir.sin() * view.radius * 1.8f32;

                collider.active = false;
                if let Some(id) = animator.playing_id() {
                    if id == CharacterAnimID::Attack {
                        collider.active = true;
                    }
                }
            });
    }
}

impl SystemProcess for System<CContainer<BodyWeaponCollider>, CContainer<CharacterView>> {
    fn process(body_weapon_colliders: &mut Self::Update, views: &Self::Refer) {
        body_weapon_colliders
            .iter_mut()
            .zip_entity(views)
            .for_each(|(_, collider, view)| {
                collider.circle.pos = view.position;
                collider.circle.radius = view.radius;
            });
    }
}

impl SystemProcess
    for System<
        CContainer<BodyDefenseCollider>,
        (
            &CContainer<CharacterView>,
            &CContainer<SwordCollider>,
            &CContainer<BodyWeaponCollider>,
            &CContainer<Team>,
        ),
    >
{
    fn process(
        body_defenses: &mut Self::Update,
        (character_views, sword_colliders, body_weapon_colliders, teams): &Self::Refer,
    ) {
        body_defenses
            .iter_mut()
            .zip_entity2(character_views, teams)
            .for_each(|(defense_entity_id, body_defense, view, defense_team)| {
                body_defense.hit = false;
                body_defense.circle.pos = view.position;
                body_defense.circle.radius = view.radius;

                sword_colliders.iter().zip_entity(teams).for_each(
                    |(sword_entity_id, sword_collider, sword_team)| {
                        if defense_entity_id == sword_entity_id {
                            return;
                        }
                        if defense_team.team_id() == sword_team.team_id() {
                            return;
                        }
                        if sword_collider.is_collided(body_defense) {
                            body_defense.hit = true;
                        }
                    },
                );

                body_weapon_colliders.iter().zip_entity(teams).for_each(|(weapon_entity_id, weapon_collider, weapon_team)|{
                    if defense_entity_id == weapon_entity_id {
                        return;
                    }
                    if defense_team.team_id() == weapon_team.team_id() {
                        return;
                    }
                    if weapon_collider.is_collided(body_defense) {
                        body_defense.hit = true;
                    }
                });
            });
    }
}

impl SystemProcess for System<CContainer<CharacterAnimator>, ()> {
    fn process(animators: &mut Self::Update, _: &Self::Refer) {
        animators
            .iter_mut()
            .for_each(|(_, animator)| animator.update());
    }
}

impl SystemProcess for System<CContainer<CharacterAnimator>, CContainer<Input>> {
    fn process(animators: &mut Self::Update, inputs: &Self::Refer) {
        animators
            .iter_mut()
            .zip_entity(inputs)
            .for_each(|(_, animator, input)| {
                if let Some(id) = animator.playing_id() {
                    if id == CharacterAnimID::Attack && animator.is_end() {
                        animator.play(CharacterAnimID::Wait);
                    }
                    if input.attack && id != CharacterAnimID::Attack {
                        animator.play(CharacterAnimID::Attack);
                    }
                }
            });
    }
}

impl SystemProcess for System<CContainer<CharacterAnimator>, CContainer<BodyDefenseCollider>> {
    fn process(animators: &mut Self::Update, defense_colliders: &Self::Refer) {
        animators
            .iter_mut()
            .zip_entity(defense_colliders)
            .for_each(|(_, animator, collider)| {
                if let Some(id) = animator.playing_id() {
                    if id == CharacterAnimID::Damaged && animator.is_end() {
                        animator.play(CharacterAnimID::Wait);
                    }
                    if collider.hit && id != CharacterAnimID::Damaged {
                        animator.play(CharacterAnimID::Damaged);
                    }
                }
            });
    }
}

impl SystemProcess for System<CContainer<CharacterView>, CContainer<CharacterAnimator>> {
    fn process(views: &mut Self::Update, animators: &Self::Refer) {
        views
            .iter_mut()
            .zip_entity(animators)
            .for_each(|(_, view, animator)| {
                if let Some(val) = animator.value() {
                    view.radius_scale = val.radius_scale;
                    view.weapon_direction = val.weapon_direction;
                }
            });
    }
}

impl SystemProcess
    for System<CContainer<CharacterView>, (&CContainer<Position>, &CContainer<Direction>)>
{
    fn process(views: &mut Self::Update, (positions, directions): &Self::Refer) {
        views
            .iter_mut()
            .zip_entity2(positions, directions)
            .for_each(|(_, view, pos, dir)| {
                view.position.x = pos.x;
                view.position.y = pos.y;
                view.direction = *dir;
            });
    }
}

impl SystemProcess for System<Window, CContainer<CharacterView>> {
    fn process(window: &mut Self::Update, views: &Self::Refer) {
        views.iter().for_each(|(_, view)| {
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

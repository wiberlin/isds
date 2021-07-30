use super::*;

use std::cmp;
use std::collections::BTreeSet;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PeerSet(pub BTreeSet<Entity>);
impl PeerSet {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
}

pub fn pick_random_peer(sim: &mut Simulation, node: Entity) -> Option<Entity> {
    let peers = peers(sim, node).0.clone(); // TODO: the `.clone()` here is not ideal
    peers.iter().choose(&mut sim.rng).copied()
}

pub fn send_message_to_random_peer<P: hecs::Component>(
    sim: &mut Simulation,
    source: Entity,
    payload: P,
) -> Result<Entity, String> {
    if let Some(dest) = pick_random_peer(sim, source) {
        Ok(sim.send_message(source, dest, payload))
    } else {
        Err("Couldn't find a suitable message destination. Not enough peers?".into())
    }
}

pub fn add_random_nodes_as_peers(
    sim: &mut Simulation,
    node: Entity,
    new_peers_min: usize,
    new_peers_max: usize,
) {
    let mut candidates = sim.all_other_nodes(node);
    let peers = peers(sim, node).clone();
    candidates.retain(|id| !peers.0.contains(id));

    let new_peers_min = cmp::min(new_peers_min, candidates.len());
    let new_peers_max = cmp::min(new_peers_max, candidates.len());
    let number_of_new_peers = sim.rng.gen_range(new_peers_min..new_peers_max);

    let new_peers = candidates.choose_multiple(&mut sim.rng, number_of_new_peers);
    for &peer in new_peers {
        add_peer(sim, node, peer);
    }
}

pub fn make_delaunay_network(sim: &mut Simulation) {
    use delaunator::{triangulate, Point};

    let (nodes, points): (Vec<Entity>, Vec<Point>) = sim
        .world
        .query_mut::<(&UnderlayNodeName, &UnderlayPosition)>()
        .into_iter()
        .map(|(id, (_, pos))| {
            (
                id,
                Point {
                    x: pos.x as f64,
                    y: pos.y as f64,
                },
            )
        })
        .unzip();

    for &node in nodes.iter() {
        *peers(sim, node) = PeerSet::new();
    }
    let triangles = triangulate(&points)
        .expect("No triangulation exists.")
        .triangles;
    assert!(triangles.len() % 3 == 0);

    for i in (0..triangles.len()).step_by(3) {
        let node1 = nodes[triangles[i]];
        let node2 = nodes[triangles[i + 1]];
        let node3 = nodes[triangles[i + 2]];
        add_peer(sim, node1, node2);
        add_peer(sim, node1, node3);
        add_peer(sim, node2, node1);
        add_peer(sim, node2, node3);
        add_peer(sim, node3, node1);
        add_peer(sim, node3, node2);
    }
}

pub fn add_peer(sim: &mut Simulation, node: Entity, peer: Entity) {
    peers(sim, node).0.insert(peer);
}

pub fn peers(sim: &mut Simulation, node: Entity) -> hecs::RefMut<PeerSet> {
    if sim.world.get_mut::<PeerSet>(node).is_err() {
        let peers = PeerSet::new();
        sim.world.insert_one(node, peers).unwrap();
    }
    sim.world.get_mut::<PeerSet>(node).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn add_peer_adds_peer() {
        let mut sim = Simulation::new();
        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        add_peer(&mut sim, node1, node2);

        let expected = PeerSet(vec![node2].into_iter().collect());
        let actual = (*sim.world.get::<PeerSet>(node1).unwrap()).clone();

        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn add_random_other_nodes_as_peers_adds_peers() {
        let mut sim = Simulation::new();
        let node1 = sim.spawn_random_node();
        sim.spawn_random_node();
        sim.spawn_random_node();
        sim.spawn_random_node();
        sim.spawn_random_node();

        add_random_nodes_as_peers(&mut sim, node1, 2, 3);

        let peers = peers(&mut sim, node1);
        let actual = peers.0.len();
        let expected_min = 2;
        let expected_max = 3;

        assert!(expected_min <= actual);
        assert!(actual <= expected_max);
        assert!(!peers.0.contains(&node1));
    }
}
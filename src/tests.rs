#[cfg(test)]
mod tests {

    #[test]
    fn load_gltf() {
        let gltf = gltf::import("./assets/basicmesh.glb").unwrap();
    }
}

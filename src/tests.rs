#[cfg(test)]
mod tests {
    use ash::vk;

    #[test]
    fn defaults() {
        let foo = vk::StencilOpState::default();
        println!("{}", foo.fail_op.as_raw());
        println!("{}", foo.pass_op.as_raw());
        println!("{}", foo.depth_fail_op.as_raw());
    }
}
#[macro_export]
macro_rules! vk_check {
    ($x:ident) => {
        let err = $x.result();
        if let Err(e) = err {
            println!("Detected Vulkan error: {e}");
            ::std::process::abort();
        };
    };
}


//Macro works, testing that something aborts is a pain in the ass
#[cfg(test)]
mod tests {
    #[test]
    fn macro_check() {
        let a = ::ash::vk::Result::SUCCESS;
        vk_check!(a);
        assert!(true);
    }
}
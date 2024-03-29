cfg_if::cfg_if! {
    if #[cfg(debug_assertions)]{
        use ash::vk;
        use std::borrow::Cow;
        use std::ffi::CStr;
    }
}
#[cfg(debug_assertions)]
pub unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!("{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n");
        }
        _ => {
            log::error!("{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n");
        }
    }

    vk::FALSE
}

//! Event types for the DOM renderer.

/// Map rye event names to DOM event names.
pub fn dom_event_name(rye_event: &str) -> &str {
    match rye_event {
        "onclick" => "click",
        "oninput" => "input",
        "onchange" => "change",
        "onsubmit" => "submit",
        "onkeydown" => "keydown",
        "onkeyup" => "keyup",
        "onkeypress" => "keypress",
        "onfocus" => "focus",
        "onblur" => "blur",
        "onmouseenter" => "mouseenter",
        "onmouseleave" => "mouseleave",
        "onmousedown" => "mousedown",
        "onmouseup" => "mouseup",
        "onmousemove" => "mousemove",
        "ontouchstart" => "touchstart",
        "ontouchend" => "touchend",
        "ontouchmove" => "touchmove",
        "onscroll" => "scroll",
        "onresize" => "resize",
        other => other,
    }
}

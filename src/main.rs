use i_slint_backend_winit::winit::window::WindowButtons;
use i_slint_backend_winit::WinitWindowAccessor;
use shadow_rs::shadow;
//use slint::Model;
use slint::Timer;
use slint::TimerMode;
//use slint::VecModel;
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

slint::include_modules!();

fn main() {
    shadow!(build);
    //Screen size
    let output = Command::new("adb")
        .args(["shell", "wm", "size"])
        .output()
        .unwrap();
    let display = String::from_utf8_lossy(&output.stdout).to_string();
    let display_size = display.trim().split(" ").collect::<Vec<_>>()[2]
        .split("x")
        .collect::<Vec<_>>();
    //println!("W:{},H:{}", display_size[0], display_size[1]);

    let drag_f = Rc::new(RefCell::new(false)); //swipe flag

    let app = main_win::new().unwrap();
    let app_weak = app.as_weak();
    app_weak.unwrap().set_appname(
        format!(
            "{} v{}_{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            build::BUILD_TIME
        )
        .into(),
    );

    //disable maximize button
    app.window()
        .with_winit_window(|winit_win: &i_slint_backend_winit::winit::window::Window| {
            winit_win.set_enabled_buttons(WindowButtons::MINIMIZE | WindowButtons::CLOSE)
        });

    let app_weak = app.as_weak();
    app_weak
        .unwrap()
        .set_d_w(display_size[0].parse::<i32>().unwrap().div_euclid(3) as f32);
    app_weak
        .unwrap()
        .set_d_h(display_size[1].parse::<i32>().unwrap().div_euclid(3) as f32);

    //handle click or  swipe event
    let app_weak = app.as_weak();
    let drag_flag = drag_f.clone();
    app.on_touch_position(move || {
        /*
        let touch_pos = app_weak.unwrap().get_list_of_position();
        let pos = touch_pos.as_any().downcast_ref::<VecModel<f32>>().unwrap();

        for i in pos.iter() {
            println!("pos:{}", i);
        }
        */
        // println!("touch:1-{}", drag_flag.borrow());
        if *drag_flag.borrow() {
            /*
                        println!(
                            "before:x:{:?},y:{:?}",
                            app_weak.unwrap().get_tx(),
                            app_weak.unwrap().get_ty()
                        );
                        println!(
                            "after:x:{:?},y:{:?}",
                            app_weak.unwrap().get_mx(),
                            app_weak.unwrap().get_my()
                        );
                        println!("touch:2-{}", drag_flag.borrow());
                        println!("drag");
            */
            Command::new("adb")
                .args([
                    "shell",
                    "input",
                    "swipe",
                    &{ app_weak.unwrap().get_tx() as i32 * 3 }.to_string(),
                    &{ app_weak.unwrap().get_ty() as i32 * 3 }.to_string(),
                    &{ app_weak.unwrap().get_mx() as i32 * 3 }.to_string(),
                    &{ app_weak.unwrap().get_my() as i32 * 3 }.to_string(),
                ])
                .status()
                .unwrap();
            println!(
                "Swipe:{} {} - {} {}",
                app_weak.unwrap().get_tx(),
                app_weak.unwrap().get_ty(),
                app_weak.unwrap().get_mx(),
                app_weak.unwrap().get_my()
            );
            *drag_flag.borrow_mut() = false;
            // println!("touch:3-{}", drag_flag.borrow());
        } else {
            Command::new("adb")
                .args([
                    "shell",
                    "input",
                    "tap",
                    &{ app_weak.unwrap().get_tx() as i32 * 3 }.to_string(),
                    &{ app_weak.unwrap().get_ty() as i32 * 3 }.to_string(),
                ])
                .status()
                .unwrap();
            println!(
                "Click:{} {}",
                app_weak.unwrap().get_tx(),
                app_weak.unwrap().get_ty()
            );
        }
    });

    //handle swipe event, set swipe flag
    let drag_flag = drag_f.clone();
    app.on_move_position(move || {
        *drag_flag.borrow_mut() = true;
        // println!("in move:{}", drag_flag.borrow());
    });

    // right click
    app.on_return_back(move || {
        println!("Right click");
        Command::new("adb")
            .args(["shell", "input", "keyevent", "KEYCODE_BACK"])
            .status()
            .unwrap();
    });

    //loop screencap
    let app_weak = app.as_weak();
    let time = Timer::default(); //need this define.
    time.start(
        TimerMode::Repeated,
        std::time::Duration::from_millis(1),
        move || {
            let output = Command::new("adb")
                .args(["exec-out", "screencap -p"])
                .output()
                .unwrap();
            let mut f = File::create("screen.png").unwrap();
            f.write_all(&output.stdout).unwrap();
            app_weak
                .upgrade_in_event_loop(|app| {
                    app.set_srcimg(slint::Image::load_from_path(Path::new("screen.png")).unwrap());
                })
                .unwrap();
        },
    );

    app.run().unwrap();
}

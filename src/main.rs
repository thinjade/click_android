use i_slint_backend_winit::winit::window::WindowButtons;
use i_slint_backend_winit::WinitWindowAccessor;
use shadow_rs::shadow;
//use slint::Model;
use slint::Timer;
use slint::TimerMode;
//use slint::VecModel;
use image::imageops::FilterType;
//use image::ImageFormat;
use image::ImageReader;
use slint::Image;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;
use std::cell::RefCell;
//use std::fs::File;
use std::io::Cursor;
//use std::io::Write;
//use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::time::Instant;

slint::include_modules!();

fn main() {
    shadow!(build);

    let con_output = Command::new("adb")
        .args(["devices"])
        .output()
        .expect("Connect wrong");
    let std_out_str = String::from_utf8(con_output.stdout).unwrap();
    let std_out: Vec<_> = std_out_str.split("\r\n").collect();

    if std_out[1].is_empty() {
        println!("No device connected, check and retry");
        let _ = Command::new("adb")
            .args(["kill-server"])
            .output()
            .expect("Connect wrong");
        println!("\nPress ENTER to quit...");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {}
    } else {
        println!("Connected: {}", std_out[1]);
        //Screen size
        let output = Command::new("adb")
            .args(["shell", "wm", "size"])
            .output()
            .unwrap();
        let display = String::from_utf8_lossy(&output.stdout).to_string();
        let display_size = display
            .trim()
            .split(" ")
            .collect::<Vec<_>>()
            .get(2)
            .unwrap()
            .split("x")
            .collect::<Vec<_>>();
        let img_w = display_size[0].parse::<i32>().unwrap().div_euclid(3);
        let img_h = display_size[1].parse::<i32>().unwrap().div_euclid(3);

        let drag_f = Rc::new(RefCell::new(false)); //drag flag

        let click_down = Rc::new(RefCell::new(Instant::now()));
        let click_up = Rc::new(RefCell::new(Instant::now()));
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
        let app_window = app.window();
        app_window.with_winit_window(|winit_win: &i_slint_backend_winit::winit::window::Window| {
            winit_win.set_enabled_buttons(WindowButtons::MINIMIZE | WindowButtons::CLOSE)
        });
        app_window.on_close_requested(|| {
            let _ = Command::new("adb")
                .args(["kill-server"])
                .output()
                .expect("Connect wrong");
            println!("Quit");
            slint::CloseRequestResponse::HideWindow
        });

        let app_weak = app.as_weak();
        app_weak.unwrap().set_d_w(img_w as f32);
        app_weak.unwrap().set_d_h(img_h as f32);

        //handle click or  swipe event
        let app_weak = app.as_weak();
        let drag_flag = drag_f.clone();
        let click_down_clone = click_down.clone();
        let click_up_clone = click_up.clone();
        app.on_left_click(move || {
            /*
            let touch_pos = app_weak.unwrap().get_list_of_position();
            let pos = touch_pos.as_any().downcast_ref::<VecModel<f32>>().unwrap();

            for i in pos.iter() {
                println!("pos:{}", i);
            }
            */
            *click_up_clone.borrow_mut() = Instant::now();
            let press_time = click_up_clone
                .borrow()
                .duration_since(*(click_down_clone.borrow()))
                .as_millis();

            if *drag_flag.borrow() {
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
            } else if press_time > 1000 {
                Command::new("adb")
                    .args([
                        "shell",
                        "input",
                        "swipe",
                        &{ app_weak.unwrap().get_tx() as i32 * 3 }.to_string(),
                        &{ app_weak.unwrap().get_ty() as i32 * 3 }.to_string(),
                        &{ app_weak.unwrap().get_tx() as i32 * 3 }.to_string(),
                        &{ app_weak.unwrap().get_ty() as i32 * 3 }.to_string(),
                        "1000",
                    ])
                    .status()
                    .unwrap();
                println!(
                    "Long press: {} {}",
                    app_weak.unwrap().get_tx(),
                    app_weak.unwrap().get_ty()
                );
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
        app.on_left_move(move || {
            *drag_flag.borrow_mut() = true;
        });

        let click_down_clone = click_down.clone();
        app.on_left_click_down(move || {
            *click_down_clone.borrow_mut() = Instant::now();
        });

        // right click
        app.on_right_click(move || {
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
                let app_weak_clone = app_weak.clone();
                let output = Command::new("adb")
                    .args(["exec-out", "screencap -p"])
                    .output()
                    .unwrap();

                let screen_image = ImageReader::new(Cursor::new(&output.stdout))
                    .with_guessed_format()
                    .unwrap();
                /*
                let mut f = File::create("screen.png").unwrap();
                f.write_all(&output.stdout).unwrap();
                app_weak
                    .upgrade_in_event_loop(|app| {
                        app.set_srcimg(slint::Image::load_from_path(Path::new("screen.png")).unwrap());
                    })
                    .unwrap();
                    */

                let image_data = screen_image.decode().unwrap().into_rgba8();
                let image_scale = image::imageops::resize(
                    &image_data,
                    img_w as u32,
                    img_h as u32,
                    FilterType::Triangle,
                );
                let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                    image_scale.as_raw(),
                    image_scale.width(),
                    image_scale.height(),
                );
                /*
                match app_weak_clone.upgrade() {
                    Some(a) => {
                        let image = Image::from_rgba8(buffer);
                        a.set_srcimg(image);
                    }
                    None => {
                        println!("set image error");
                    }
                }
                */

                app_weak_clone
                    .upgrade_in_event_loop(|app| {
                        let image = Image::from_rgba8_premultiplied(buffer);
                        app.set_srcimg(image);
                    })
                    .unwrap();
            },
        );

        app.run().unwrap();
    }
}

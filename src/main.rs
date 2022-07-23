use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

const SQUARES: i16 = 8;
type Point = (i16, i16);

const DR: f32 = 0.0174533;

pub struct Player {
    pub x_pos: f32,
    pub y_pos: f32,
    pub size: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub angle: f32,
    pub color: Color,
}

fn window_config() -> Conf {
    Conf { 
        window_title: "raymond-caster".to_owned(), 
        window_width: 1024, 
        window_height: 512, 
        ..Default::default()
        // high_dpi: (), 
        // fullscreen: (), 
        // sample_count: (), 
        // window_resizable: (), 
        // icon: (), 
        // platform: () 
    }
}

#[macroquad::main(window_config)]
async fn main() {
    // let mut last_update = get_time();
    // let speed = 0.3;
    let map: [u8; 8 * 8] = [
        1, 1, 1, 1, 1, 1, 1, 1,
        1, 0, 1, 0, 0, 0, 0, 1,
        1, 0, 1, 0, 0, 0, 0, 1,
        1, 0, 1, 0, 0, 0, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 0, 0, 0, 1, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1,
        1, 1, 1, 1, 1, 1, 1, 1,
    ];

    let mut player = Player {
        x_pos: 300.0,
        y_pos: 300.0,
        size: 10.0,
        delta_x: 0.0_f32.cos() * 5.,
        delta_y: 0.0_f32.sin() * 5.,
        angle: 0.0,
        color: Color::from_rgba(255, 255, 0, 255),
    };

    loop {
        clear_background(DARKGRAY);

        // Draw map
        let mut square_color = DARKGRAY;

        for y in 0..8 {
            for x in 0..8 {
                if map[(y * 8 + x) as usize] == 1 {
                    square_color = WHITE;
                } else { 
                    square_color = BLACK;
                }

                let scaled_x = x as f32 * 64.;
                let scaled_y = y as f32 * 64.;

                draw_rectangle(
                    scaled_x + 1., 
                    scaled_y + 1.,
                    64. - 1.,
                    64. - 1.,
                    square_color
                );
            }
        }



        // Handle input
        let movement_speed = 5.;
        if is_key_down(KeyCode::W) {
            player.y_pos += player.delta_y;
            player.x_pos += player.delta_x;
        } else if is_key_down(KeyCode::A) {
            player.angle -= 0.05;

            if player.angle < 0. {
                player.angle += TAU;
            }

            player.delta_x = player.angle.cos() * movement_speed;
            player.delta_y = player.angle.sin() * movement_speed;            
        } else if is_key_down(KeyCode::S) {
            player.y_pos -= player.delta_y;
            player.x_pos -= player.delta_x;
        } else if is_key_down(KeyCode::D) {
            player.angle += 0.05;

            if player.angle > TAU {
                player.angle -= TAU;
            }

            player.delta_x = player.angle.cos() * movement_speed;
            player.delta_y = player.angle.sin() * movement_speed;
        }

        // Draw player
        draw_rectangle(
            player.x_pos - (player.size / 2.), 
            player.y_pos - (player.size / 2.), 
            player.size, 
            player.size, 
            player.color
        );

        draw_line(
            player.x_pos,
            player.y_pos, 
            player.x_pos + player.delta_x * movement_speed, 
            player.y_pos + player.delta_y * movement_speed,
            3.0,
            player.color
        );


        // Calculate Ray Casting
        let mut ray_angle = player.angle - DR * 30.;

        if ray_angle < 0. {
            ray_angle += TAU;
        }

        if ray_angle > TAU {
            ray_angle -= TAU;
        }

        let mut y_offset = 0.0;
        let mut x_offset = 0.0;
        let mut ray_x = 0.0;
        let mut ray_y = 0.0;
        let mut map_x: i32 = 0;
        let mut map_y: i32 = 0;
        let mut map_pos: usize = 0;
        let ray_count = 60;
        let mut distance_t = 0.0;
        
        for ray in 0..ray_count {
            let mut depth_of_field = 0;
            let mut distance_horizontal = 1_000_000.0;
            let mut hori_x = player.x_pos;
            let mut hori_y = player.y_pos;

            // Check horizontal grid lines
            let arc_tan = -1.0 / ray_angle.tan();

            if ray_angle > PI { // looking up
                ray_y = (((player.y_pos as i32) >> 6) << 6) as f32 - 0.0001;
                ray_x = (player.y_pos - ray_y) * arc_tan + player.x_pos;
                y_offset = -64.;
                x_offset = -y_offset * arc_tan;
            }

            if ray_angle < PI { // looking down
                ray_y = (((player.y_pos as i32) >> 6) << 6) as f32 + 64.;
                ray_x = (player.y_pos - ray_y) * arc_tan + player.x_pos;
                y_offset = 64.;
                x_offset = -y_offset * arc_tan;
            }

            if ray_angle == 0.0 || ray_angle == PI { // looking straight left or right
                ray_x = player.x_pos;
                ray_y = player.y_pos;
                depth_of_field = 8;
            }

            while depth_of_field < 8 {
                map_x = (ray_x as i32) >> 6;
                map_y = (ray_y as i32) >> 6;
                map_pos = (map_y * 8 + map_x) as usize;

                if map_pos > 0 && map_pos < 64 && map[map_pos] == 1 {
                    hori_x = ray_x;
                    hori_y = ray_y;
                    distance_horizontal = distance(player.x_pos, player.y_pos, hori_x, hori_y, ray_angle as f32);
                    depth_of_field = 8; // Ray hit wall.
                } else {
                    ray_x += x_offset;
                    ray_y += y_offset;
                    depth_of_field += 1;
                }
            }

            let mut depth_of_field = 0;
            let mut distance_vertical = 1_000_000.0;
            let mut vert_x = player.x_pos;
            let mut vert_y = player.y_pos;
            // Check vertical grid lines
            let neg_tan = -(ray_angle.tan());

            if ray_angle > PI / 2. && ray_angle < 3. *PI / 2. { // looking left
                ray_x = (((player.x_pos as i32) >> 6) << 6) as f32 - 0.0001;
                ray_y = (player.x_pos - ray_x) * neg_tan + player.y_pos;
                x_offset = -64.;
                y_offset = -x_offset * neg_tan;
            }

            if ray_angle < PI / 2. || ray_angle > 3. * PI / 2. { // looking right
                ray_x = (((player.x_pos as i32) >> 6) << 6) as f32 + 64.;
                ray_y = (player.x_pos - ray_x) * neg_tan + player.y_pos;
                x_offset = 64.;
                y_offset = -x_offset * neg_tan;
            }

            if ray_angle == 0.0 || ray_angle == PI { // looking straight up or down
                ray_x = player.x_pos;
                ray_y = player.y_pos;
                depth_of_field = 8;
            }

            while depth_of_field < 8 {
                map_x = (ray_x as i32) >> 6;
                map_y = (ray_y as i32) >> 6;
                map_pos = (map_y * 8 + map_x) as usize;

                if map_pos > 0 && map_pos < 64 && map[map_pos] == 1 {
                    vert_x = ray_x;
                    vert_y = ray_y;
                    distance_vertical = distance(player.x_pos, player.y_pos, vert_x, vert_y, ray_angle as f32);
                    depth_of_field = 8; // Ray hit wall.
                } else {
                    ray_x += x_offset;
                    ray_y += y_offset;
                    depth_of_field += 1;
                }
            }

            // Set the length of the casted ray to the shortest horizontal or vertical measurement.
            let mut wall_color = RED;
            if distance_vertical < distance_horizontal {
                ray_x = vert_x;
                ray_y = vert_y;
                distance_t = distance_vertical;
                wall_color = Color::from_rgba(200, 0, 0, 255);
            }

            if distance_horizontal < distance_vertical {
                ray_x = hori_x;
                ray_y = hori_y;
                distance_t = distance_horizontal;
                wall_color = Color::from_rgba(175, 0, 0, 255);
            }

            // Draw the Ray
            draw_line(
                player.x_pos,
                player.y_pos,
                ray_x,
                ray_y,
                1.0,
                RED
            );

            // Draw 3D walls
            let mut c_angle = player.angle - ray_angle;

            if  c_angle < 0.0 {
                c_angle += TAU;
            }

            if c_angle > TAU {
                c_angle -= TAU;
            }

            distance_t = distance_t * c_angle.cos(); // fixes fish-eye effect

            let mut line_height = (64. * 320.) / distance_t;

            if line_height > 320. {
                line_height = 320.0;
            }

            let line_offset = 160.0 - line_height / 2.0;

            draw_line(
                ray as f32 * 8.0 + 530.0,
                line_offset,
                ray as f32 * 8.0 + 530.0,
                line_height + line_offset,
                8.0,
                wall_color
            );

            // update for next ray
            ray_angle += DR;

            if ray_angle < 0. {
                ray_angle += TAU;
            }
    
            if ray_angle > TAU {
                ray_angle -= TAU;
            }
        }
                
        
        next_frame().await
    }
}

pub fn distance(ax: f32, ay: f32, bx: f32, by: f32, _angle: f32) -> f32 {
    ((bx - ax) * (bx - ax) + (by - ay) * (by - ay)).sqrt()   
}
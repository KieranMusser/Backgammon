use raylib::prelude::*;
use rand::prelude::*;

use raylib::ffi::SetConfigFlags;
use raylib::ffi::ConfigFlags::FLAG_VSYNC_HINT;


const TRI_WIDTH: f32 = 50.0;

#[derive(Copy, Clone, Debug)]
struct Cell {
	black: bool,
	pieces: i32
}

fn draw_piece(d: &mut RaylibDrawHandle, piece_num: i32, x_pos: i32, y_flip: bool, black: bool) {
	let y_pos = (20.0 + piece_num as f32 * 40.0) as i32;
	d.draw_circle(
			x_pos, 
			if y_flip { 480 - y_pos } else { y_pos },
			20.0,
			if black { Color::BLACK } else { Color::WHITE }
			);
}

fn draw_board(d: &mut RaylibDrawHandle, board: &[Cell], bar: &[Cell], black: bool, raw_selection: i32) {
	let selection = if black { raw_selection } else { 23 - raw_selection };
	for x in 0..2 {
		let x_shift = x as f32 * 340.0;
		for n in 0..6 {
			let mut dcol = if n % 2 == 0 { Color::DARKBROWN } else { Color::BROWN };
			let mut upcol = if n % 2 == 1 { Color::DARKBROWN } else { Color::BROWN };
			if selection as usize == x * 6 + n {
				dcol = Color::YELLOW;
			}
			if selection as usize == 23 -  (x * 6 + n) {
				upcol = Color::YELLOW;
			}
			d.draw_triangle(
					Vector2 { x: TRI_WIDTH * n as f32 + x_shift, y: 0.0 },
					Vector2 { x: TRI_WIDTH * n as f32 + TRI_WIDTH / 2.0 + x_shift, y: 200.0 },
					Vector2 { x: TRI_WIDTH * n as f32 + TRI_WIDTH +  x_shift, y: 0.0 },
					dcol
					);

			d.draw_triangle(
					Vector2 { x: TRI_WIDTH * n as f32 + TRI_WIDTH / 2.0 + x_shift, y: 280.0 },
					Vector2 { x: TRI_WIDTH * n as f32 + x_shift, y: 480.0 },
					Vector2 { x: TRI_WIDTH * n as f32 + TRI_WIDTH +  x_shift, y: 480.0 },
					upcol
					);


			let cur_col = x * 6 + n;
			let low = if black { cur_col } else { 23 - cur_col };
			let up = if black { 23 - cur_col } else { cur_col };

			for c in 0..board[low].pieces {
				draw_piece(d, c, (TRI_WIDTH * (n as f32 + 0.5) + x_shift) as i32, false, board[low].black);
			}

			for c in 0..board[up].pieces {
				draw_piece(d, c, (TRI_WIDTH * (n as f32 + 0.5) + x_shift) as i32, true, board[up].black);
			}
		}
	}

	/* Draw bar */
	d.draw_rectangle_v(
		Vector2 {x: 300.0, y: 0.0},
		Vector2 {x: 40.0, y: 480.0},
		if raw_selection == 0xBA4 { Color::YELLOW } else {Color::DARKBROWN}
	);
	for p in 0..2 {
		for c in 0..bar[p].pieces {
			draw_piece(d,c,320, bar[p].black ^ black, bar[p].black);
		}
	}
}

fn handle_click(mouse: Vector2, black: bool) -> i32 {
	let mut index: i32 = 0;
	if mouse.x > 340.0 {
		index += ((mouse.x - 340.0) / TRI_WIDTH) as i32;
		index += 6;
	} else if mouse.x < 300.0 {
		index += (mouse.x / TRI_WIDTH) as i32;
	} else {
		return 0xBA4; /* magic value, on bar */
	}
	if mouse.y > 240.0 {
		index = 23 - index;
	}
	if !black {  index = 23 - index } ;
	return index;
}

fn move_off_bar(board: &mut [Cell], bar: &mut [Cell], black: bool, end: i32) -> bool {
	let bi: usize = if black { 1 } else { 0 };
	if black && (end < 0 || end > 5) {
		return false;
	} else if !black && (end < 17 || end > 23) {
		return false;
	}
	if board[end as usize].black != black {
		if board[end as usize].pieces > 1 { return false; }
		if board[end as usize].pieces == 1 {
			/* capture, add to other bar */
			bar[1 - bi].pieces += 1;
		}
		board[end as usize].pieces = 0;
	}
	if bar[bi].pieces == 0 {
		return false;
	}
	bar[bi].pieces -= 1;
	board[end as usize].pieces += 1;
	board[end as usize].black = black;
	return true;
}

fn move_piece(board: &mut [Cell], bar: &mut [Cell], turn_black: bool, start: i32, end: i32) -> bool {
	/* can't move back */
	if turn_black && end <= start {
		return false;
	}
	if !turn_black && end >= start {
		return false;
	}
	/* out of bounds */
	if end < 0 || end >= 24 {
		return false;
	}
	let endi = end as usize;
	let starti = start as usize;

	/* is piece at start */
	if board[starti].pieces == 0 {
		return false
	}
	/* attacking enemy spot */
	if board[endi].black != board[starti].black && board[endi].pieces <= 1 {
		/* capturing */
		if board[endi].pieces == 1 {
			if board[endi].black {
				bar[1].pieces += 1;
			} else {
				bar[0].pieces += 1;
			}
		}
		board[endi].pieces = 1;
		board[starti].pieces -= 1;
		board[endi].black = board[starti].black;
		return true;
	}
	/* moving to own stack */
	if board[endi].black == board[starti].black {
		board[endi].pieces += 1;
		board[starti].pieces -= 1;
		return true;
	}
	return false;
}

/* returns whether turn has ended */
fn take_roll(roll: &mut [i32], double_count: &mut i32, sel: i32) -> bool {
	if roll[0] == roll[1] && sel == roll[0] {
		*double_count -= 1;
		if *double_count == 0 {
			roll[0] = -1;
			roll[1] = -1;
			return true;
		} else {
			return false;
		}
	}
	if sel == roll[0] {
		roll[0] = -1;
	} else if sel == roll[1] {
		roll[1] = -1;
	}
	return roll[0] == -1 && roll[1] == -1;
}

fn can_move(roll: &[i32], sel: i32) -> bool {
	return sel == roll[0] || sel == roll[1];
}

fn reroll(roll: &mut [i32], double_count: &mut i32) {
	let mut rng = rand::thread_rng();
	let roll1: f64 = rng.gen();
	roll[0] = (roll1 * 6.0) as i32 + 1;
	let roll2: f64 = rng.gen();
	roll[1] = (roll2 * 6.0) as i32 + 1;
	if roll[0] == roll[1] {
		*double_count = 4;
	}
}

fn run_turn(roll: &mut [i32], double_count: &mut i32, turn_black: &mut bool, f_sel: i32) {
	if take_roll(roll, double_count, f_sel)  {
		*turn_black = ! *turn_black;
		reroll(roll, double_count);
	}
}

/* handle_right_click
 * board: array of Cells
 * roll: array of the 2 dice rolls
 * double_count: if rolled doubles, counts number of double rolls left
 * turn_black: whether the current turn is black
 * sel: non-relative board position selected
 */
fn handle_right_click(board: &mut [Cell], roll: &mut [i32], double_count: &mut i32, turn_black: &mut bool, bar: &[Cell], sel: i32) {


	/* Can't take pieces off if you have pieces on the bar */
	if !*turn_black && bar[0].pieces > 0 {
		return;
	}
	if *turn_black && bar[1].pieces > 0 {
		return;
	}

	for n in 0..23 {
		if ((*turn_black && n < 18) || (!*turn_black && n > 5) ) && board[n].pieces > 0 && board[n].black == *turn_black{
			return;
		}
	}
	let roll_amt = if !*turn_black { sel + 1} else { 24 - sel };
	let real_sel = (if *turn_black { sel } else { sel }) as usize;
	let mut max = 6;
	if *turn_black {
		for n in 17..=23 {
			if board[n].black && board[n].pieces > 0 {
				max = 24 - n as i32;
				break;
			}
		}
	} else {
		for n in (0..6).rev() {
			if !board[n].black && board[n].pieces > 0 {
				max = 1+ n as i32;
				break;
			}
		}
	}

	if board[real_sel].pieces == 0 || board[real_sel].black != *turn_black  {
		return;
	}
	let target : i32;
	if roll[0] >= max && max == roll_amt {
		target = roll[0];
	} else if roll[1] >= max && max == roll_amt {
		target = roll[1];
	} else {
		return;
	}
	run_turn(roll, double_count, turn_black, target);
	board[real_sel].pieces -= 1;
}


fn check_win(board: &[Cell], black: bool) -> bool {
	for cell in board {
		if cell.black == black && cell.pieces > 0 {
			return false;
		}
	}
	return true;
}


fn main() {
	let mut rl: RaylibHandle;
	let thread: RaylibThread;
	let mut board: [Cell; 24] = [
		Cell {black: true, pieces: 0 }; 24
	];
	let mut cur_sel: i32 = -1;

	/* has to be white, black */
	let mut bar: [Cell; 2] = [
		Cell {black: false, pieces: 0},
		Cell {black: true, pieces: 0}
	];

	let mut roll: [i32; 2] = [-1, -1];
	let mut turn_black: bool = true;
	let mut double_count = 0;

	reroll(&mut roll, &mut double_count);

	board[0] = Cell { black: true, pieces: 2 };
	board[5] = Cell { black: false, pieces: 5 };
	
	board[23] = Cell { black: false, pieces: 2 };
	board[18] = Cell { black: true, pieces: 5 };

	board[7] = Cell { black: false, pieces: 3 };
	board[11] = Cell { black: true, pieces: 5 };

	board[16] = Cell { black: true, pieces: 3 };
	board[12] = Cell { black: false, pieces: 5 };


//	board[23] = Cell { black: true, pieces: 4 } ;
//	board[0] = Cell { black: false, pieces: 2 } ;

	unsafe {
		SetConfigFlags(FLAG_VSYNC_HINT as u32);
		ffi::SetTraceLogLevel(ffi::TraceLogLevel::LOG_ERROR as i32);
	}

	(rl, thread) = raylib::init()
		.size(640, 480)
		.title("Backgammon")
		.build();

	rl.set_target_fps(30u32);

	while !rl.window_should_close() {
		let mut d: RaylibDrawHandle = rl.begin_drawing(&thread);

		if d.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
			let sel = handle_click(d.get_mouse_position(), turn_black);
			if cur_sel != -1 && cur_sel != sel {
				if cur_sel == 0xBA4 {
					let f_sel = if turn_black { sel + 1 } else { 24 - sel };
					if can_move(&roll, f_sel)  {
						if move_off_bar(&mut board, &mut bar, turn_black, sel) {
							run_turn(&mut roll, &mut double_count, &mut turn_black, f_sel);
						}
					}
				} else if bar[if turn_black { 1 } else { 0 }].pieces == 0 &&  can_move(&roll, (sel-cur_sel).abs()) {
					if move_piece(&mut board, &mut bar, turn_black, cur_sel, sel) {
							run_turn(&mut roll, &mut double_count, &mut turn_black, (sel-cur_sel).abs());
					}
				}
				cur_sel = -1;
			} else if cur_sel == sel {
				cur_sel = -1;
			} else if sel == 0xBA4 || (board[sel as usize].black == turn_black && board[sel as usize].pieces > 0) {
				cur_sel = sel;
			}
		}
		if d.is_mouse_button_pressed(MouseButton::MOUSE_RIGHT_BUTTON) {
			let sel = handle_click(d.get_mouse_position(), turn_black);
			let current_turn_black = turn_black;
			handle_right_click(&mut board, &mut roll, &mut double_count, &mut turn_black, &bar, sel);
			if check_win(&board, current_turn_black) {
				println!("{} wins!", if current_turn_black { "Black" } else { "White" });
				break;
			}
		}
		if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
			turn_black = ! turn_black;
			reroll(&mut roll, &mut double_count);
		}

		d.clear_background(Color::BEIGE);
		d.draw_text("Home", 12, 240, 20, Color::BLACK);
		let turn_col = if turn_black { Color::BLACK } else { Color::WHITE };
		if roll[0] == roll[1] {
			d.draw_text(format!("Roll {} ({})",roll[0],double_count).as_str(), 400, 240, 20, turn_col);
		} else {
			let r1 = if roll[0] > 0 { roll[0] } else { 0 };
			let r2 = if roll[1] > 0 { roll[1] } else { 0 };
			d.draw_text(format!("Roll {} {}",r1,r2).as_str(), 400, 240, 20, turn_col);

		}
		draw_board(&mut d, &board, &bar, turn_black, cur_sel);
	}
}

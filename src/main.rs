use proconio::{input, marker::Chars};
use std::{fmt, usize, vec};

const N_MAX: usize = 30;
const V_MAX: usize = 15;
const MAX_OPERATION_TURN: usize = 100_000;

#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

struct Operation {
    arm_move: char,
    node_rotate: Vec<char>,
    node_interact: Vec<char>,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        assert!(".LRUD".contains(self.arm_move));
        assert_eq!(self.node_interact.len(), self.node_rotate.len());
        let str_node_rotate: String = self.node_rotate[1..].iter().collect();
        let str_nodes_interact: String = self.node_interact.iter().collect();
        write!(f, "{}{}{}", self.arm_move, str_node_rotate, str_nodes_interact)
    }
}

struct BoardState {
    n: usize,                      // 盤面サイズ(n * n)
    m: usize,                      // たこ焼きの数
    takoyaki_position: Vec<Point>, // 各たこ焼きの位置
    cur_board: Vec<Vec<i32>>,      // 現在の盤面情報, 便宜的にたこ焼きに番号をつける, -1は存在しない
    final_board: Vec<Vec<bool>>,   // 最終的に目指すべき盤面
}

fn pretty_print_board_row(row: &Vec<i32>) -> String {
    let mut s = row
        .iter()
        .map(|&num| {
            if num < 10 {
                format!(" {}", num)
            } else {
                format!("{}", num)
            }
        })
        .collect::<String>();
    s.push('\n');
    s
}

impl fmt::Debug for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_board: String = self.cur_board.iter().map(pretty_print_board_row).collect();
        write!(f, "size: {} * {}\n{}", self.n, self.n, str_board)
    }
}

struct GameState {
    operations: Vec<Operation>,
    arm_state: SingleNodeArmState,
    board_state: BoardState,
    arrived_count: usize,
}

impl GameState {
    fn is_finished(&self) -> bool {
        eprintln!("{} {}", self.arrived_count, self.board_state.m);
        self.arrived_count == self.board_state.m
    }
}

struct SingleNodeArmState {
    root_position: Point,
    init_root_position: Point,
    have_takoyaki: bool,
}

impl fmt::Display for SingleNodeArmState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{} {}",
            1, self.init_root_position.x, self.init_root_position.y
        )
    }
}

#[allow(dead_code)]
struct ArmState {
    x: i32,                     // アームの根の現在位置のx座標
    y: i32,                     // アームの根の現在位置のy座標
    adj: Vec<Vec<i32>>,         // 木の頂点同士の接続情報
    edge_length: Vec<Vec<i32>>, // 辺(v, u)の長さ(1 ~ N-1)
    node_position: Vec<Point>,  // 頂点vの位置情報(x, y)
    have_takoyaki: Vec<bool>,   // 頂点vがたこ焼きを持ってるか
}

fn input_parser() -> (usize, usize, usize, BoardState) {
    input! {
        n: usize, // 盤面のサイズ
        m: usize, // たこ焼きの数
        v: usize, // ロボットアームの頂点数
    };
    assert!(n <= N_MAX);
    assert!((n * n + 9) / 10 <= m && m <= n * n / 2);
    assert!(v <= V_MAX);

    // 初期盤面の読み込み
    let mut takoyaki_position = vec![];
    let mut init_board = vec![vec![-1_i32; n]; n];
    let mut num = 0_i32;
    for i in 0..n {
        input! { si: Chars };
        assert_eq!(si.len(), n);
        for j in 0..n {
            if si[j] == '1' {
                num += 1;
                takoyaki_position.push(Point {
                    x: i as i32,
                    y: j as i32,
                });
                init_board[i][j] = num;
            }
        }
    }
    // eprintln!("{:?}", init_board);
    assert_eq!(takoyaki_position.len(), m);
    assert_eq!(num, m as i32);

    // 最終盤面の読み込み
    let mut final_board = vec![vec![false; n]; n];
    for i in 0..n {
        input! { si: Chars };
        assert_eq!(si.len(), n);
        for j in 0..n {
            if si[j] == '1' {
                final_board[i][j] = true;
            }
        }
    }
    // eprintln!("{:?}", final_board);

    let board_state = BoardState {
        n,
        m,
        takoyaki_position,
        cur_board: init_board,
        final_board,
    };

    (n, m, v, board_state)
}

fn count_arrived_takoyaki(board_state: &BoardState) -> usize {
    let mut arrived_count: usize = 0;

    let n = board_state.n;

    for i in 0..n {
        for j in 0..n {
            if board_state.cur_board[i][j] > -1 && board_state.final_board[i][j] {
                arrived_count += 1;
            }
        }
    }

    arrived_count
}

fn find_nearest_takoyaki_index_from_root(game_state: &GameState) -> usize {
    let mut idx_nearest_takoyaki = None;
    let mut min_dist = i32::MAX;

    for (i, p) in game_state.board_state.takoyaki_position.iter().enumerate() {
        if game_state.board_state.final_board[p.x as usize][p.y as usize] {
            // 既に目的地にあるたこ焼きは対象にしない
            continue;
        }
        let dist = calc_dist(p, &game_state.arm_state.root_position);
        if dist < min_dist {
            min_dist = dist;
            idx_nearest_takoyaki = Some(i);
        }
    }

    assert!(idx_nearest_takoyaki.is_some());
    eprintln!("idx_nearest_takoyaki: {}", idx_nearest_takoyaki.unwrap());
    eprintln!("min_dist: {}", min_dist);
    idx_nearest_takoyaki.unwrap()
}

fn calc_dist(p: &Point, q: &Point) -> i32 {
    (p.x - q.x).abs() + (p.y - q.y).abs()
}

/// 点pから点qにあるたこ焼きをinteractしにいく操作
fn create_operations_p_to_q(game_state: &mut GameState, q: Point) -> Vec<Operation> {
    let mut operations: Vec<Operation> = vec![];
    let p = game_state.arm_state.root_position;

    let x_dif = (q.x - p.x).abs();
    for _ in 0..x_dif {
        operations.push(Operation {
            arm_move: if p.x < q.x { 'D' } else { 'U' },
            node_rotate: vec!['.'],
            node_interact: vec!['.'],
        })
    }
    let y_dif = (q.y - p.y).abs();
    for _ in 0..y_dif {
        operations.push(Operation {
            arm_move: if p.y < q.y { 'R' } else { 'L' },
            node_rotate: vec!['.'],
            node_interact: vec!['.'],
        })
    }

    // rootのいる位置にたこ焼きがあった場合、動かずとも取らなければいけない
    if operations.is_empty() {
        operations.push(Operation {
            arm_move: '.',
            node_rotate: vec!['.'],
            node_interact: vec!['.'],
        });
    }

    let idx_last = operations.len();
    operations[idx_last - 1].node_interact[0] = 'P';

    operations
}

/// 現在のrootから最も近いたこ焼きが埋まっていない目的地を探す
fn find_nearest_vacant(game_state: &GameState) -> Point {
    let n = game_state.board_state.n;
    let mut nearest_point = None;
    let mut min_dist = i32::MAX;

    for i in 0..n {
        for j in 0..n {
            if game_state.board_state.final_board[i][j] && game_state.board_state.cur_board[i][j] == -1 {
                let dist = calc_dist(
                    &game_state.arm_state.root_position,
                    &Point {
                        x: i as i32,
                        y: j as i32,
                    },
                );
                if dist < min_dist {
                    min_dist = dist;
                    nearest_point = Some(Point {
                        x: i as i32,
                        y: j as i32,
                    });
                }
            }
        }
    }

    assert!(nearest_point.is_some());
    nearest_point.unwrap()
}

fn output_answer(game_state: &GameState) {
    // Armの出力
    println!("{}", game_state.arm_state);

    // 操作列の出力
    assert!(game_state.operations.len() <= MAX_OPERATION_TURN);
    for operation in &game_state.operations {
        println!("{}", operation);
    }
}

fn main() {
    let (_n, _m, _v, board_state) = input_parser();
    let arrived_count = count_arrived_takoyaki(&board_state);

    let mut game_state = GameState {
        operations: vec![],
        arm_state: SingleNodeArmState {
            root_position: Point { x: 0, y: 0 }, // とりあえず(0, 0)から始める
            init_root_position: Point { x: 0, y: 0 },
            have_takoyaki: false,
        },
        board_state,
        arrived_count,
    };

    while !game_state.is_finished() {
        // rootから最短のたこ焼きを探す
        let idx_nearest_takoyaki = find_nearest_takoyaki_index_from_root(&game_state);
        let q = game_state.board_state.takoyaki_position[idx_nearest_takoyaki];
        // rootから最短のたこ焼きを取りにいく操作を作る
        let mut ops = create_operations_p_to_q(&mut game_state, q);
        game_state.operations.append(&mut ops);

        // rootの位置を更新し、たこ焼きを持ってる
        game_state.arm_state.root_position = game_state.board_state.takoyaki_position[idx_nearest_takoyaki];
        game_state.arm_state.have_takoyaki = true;

        // 現在のrootから最も近い目的地を見つける
        let target_point = find_nearest_vacant(&game_state);
        // そこにたこ焼きを持っていく操作を作る
        ops = create_operations_p_to_q(&mut game_state, target_point);
        game_state.operations.append(&mut ops);

        // rootの位置を更新
        game_state.arm_state.root_position = target_point;
        game_state.arm_state.have_takoyaki = false;

        // 移動させられたたこ焼きの位置情報を更新
        let pre_takoyaki_pos = game_state.board_state.takoyaki_position[idx_nearest_takoyaki];
        game_state.board_state.cur_board[pre_takoyaki_pos.x as usize][pre_takoyaki_pos.y as usize] = -1;
        game_state.board_state.cur_board[target_point.x as usize][target_point.y as usize] =
            idx_nearest_takoyaki as i32;
        game_state.board_state.takoyaki_position[idx_nearest_takoyaki] = target_point;

        eprintln!("{:?}", game_state.board_state);
        game_state.arrived_count += 1;
    }

    output_answer(&game_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_operation_display() {
        let op = Operation {
            arm_move: 'L',
            node_rotate: vec!['.', 'L', 'R'],
            node_interact: vec!['.', '.', 'P'],
        };
        assert_eq!(format!("{op}"), "LLR..P",);

        let op = Operation {
            arm_move: '.',
            node_rotate: vec!['.'],
            node_interact: vec!['P'],
        };
        assert_eq!(format!("{op}"), ".P");
    }

    #[test]
    fn test_count_arrived() {
        let board_state = BoardState {
            n: 4,
            m: 4,
            takoyaki_position: vec![
                Point { x: 0, y: 0 },
                Point { x: 3, y: 1 },
                Point { x: 1, y: 2 },
                Point { x: 0, y: 3 },
            ],
            cur_board: vec![
                vec![0, -1, -1, 3],
                vec![-1, -1, 2, -1],
                vec![-1, -1, -1, -1],
                vec![-1, 1, -1, -1],
            ],
            final_board: vec![
                vec![false, false, false, true],
                vec![false, false, true, true],
                vec![false, false, true, false],
                vec![false, false, false, false],
            ],
        };
        assert_eq!(count_arrived_takoyaki(&board_state), 2);
    }

    #[test]
    fn test_calc_dist() {
        let p = Point { x: 3, y: 2 };
        let q = Point { x: 0, y: 5 };

        assert_eq!(calc_dist(&p, &q), 6);

        let p = Point { x: 0, y: 0 };
        let q = Point { x: 0, y: 0 };
        assert_eq!(calc_dist(&p, &q), 0);
    }
}

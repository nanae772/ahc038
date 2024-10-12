use proconio::{input, marker::Chars};
use rand::prelude::*;
use std::collections::HashMap;
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
        write!(
            f,
            "{}{}{}",
            self.arm_move, str_node_rotate, str_nodes_interact
        )
    }
}

struct BoardState {
    n: usize,                    // 盤面サイズ(n * n)
    m: usize,                    // たこ焼きの数
    cur_board: Vec<Vec<bool>>,   // 現在の盤面情報, 便宜的にたこ焼きに番号をつける, -1は存在しない
    final_board: Vec<Vec<bool>>, // 最終的に目指すべき盤面
}

impl BoardState {
    fn is_in_board(&self, x: i32, y: i32) -> bool {
        0 <= x && x < self.n as i32 && 0 <= y && y < self.n as i32
    }
}

fn pretty_print_board_row(row: &Vec<bool>) -> String {
    let mut s = row
        .iter()
        .map(|&b| if b { "1" } else { "0" })
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
    arm_state: StarArmState,
    board_state: BoardState,
    arrived_count: usize,
    mode_catch: bool,
}

impl GameState {
    /// ゲームの終了判定
    fn is_finished(&self) -> bool {
        // eprintln!("{} {}", self.arrived_count, self.board_state.m);
        self.operations.len() == MAX_OPERATION_TURN || self.arrived_count == self.board_state.m
    }
    /// rootから最短の目的地にいないたこ焼きを見つける
    fn find_nearest_takoyaki(&self) -> Point {
        let mut min_dist = i32::MAX;
        let mut nearest_point = None;

        let n = self.board_state.n;

        for i in 0..n {
            for j in 0..n {
                if self.board_state.cur_board[i][j] && !self.board_state.final_board[i][j] {
                    let p = Point {
                        x: i as i32,
                        y: j as i32,
                    };
                    let dist = calc_dist(&p, &self.arm_state.root);
                    if dist < min_dist {
                        min_dist = dist;
                        nearest_point = Some(p);
                    }
                }
            }
        }
        assert!(nearest_point.is_some());
        nearest_point.unwrap()
    }

    /// rootから最短の空き目的地を見つける
    fn find_nearest_vacant(&self) -> Point {
        let mut min_dist = i32::MAX;
        let mut nearest_point = None;

        let n = self.board_state.n;

        for i in 0..n {
            for j in 0..n {
                if !self.board_state.cur_board[i][j] && self.board_state.final_board[i][j] {
                    let p = Point {
                        x: i as i32,
                        y: j as i32,
                    };
                    let dist = calc_dist(&p, &self.arm_state.root);
                    if dist < min_dist {
                        min_dist = dist;
                        nearest_point = Some(p);
                    }
                }
            }
        }
        assert!(nearest_point.is_some());
        nearest_point.unwrap()
    }
    /// 現在のrootの位置から合法なrootの動きを返す
    fn get_legal_root_move(&self) -> Vec<char> {
        let mut moves = vec![];
        let direction_dif =
            HashMap::from([('U', (-1, 0)), ('D', (1, 0)), ('L', (0, -1)), ('R', (0, 1))]);
        let n = self.board_state.n as i32;
        for (dir, dif) in direction_dif {
            let x = self.arm_state.root.x + dif.0;
            let y = self.arm_state.root.y + dif.1;
            if 0 <= x && x < n && 0 <= y && y < n {
                moves.push(dir);
            }
        }
        assert!(!moves.is_empty());
        moves
    }

    // 点(x, y)にキャッチすべきたこ焼きがあるかどうか判定
    fn should_catch_takoyaki(&self, x: i32, y: i32) -> bool {
        // 盤外にあるのは弾く
        if !self.board_state.is_in_board(x, y) {
            return false;
        }
        // たこ焼きがあり、かつそこが目的地でないときのみ拾うべき
        self.board_state.cur_board[x as usize][y as usize]
            && !self.board_state.final_board[x as usize][y as usize]
    }

    /// 点(x, y)にたこ焼きをリリースすべきか判定
    fn should_release_takoyaki(&self, x: i32, y: i32) -> bool {
        // 盤外にあるのは弾く
        if !self.board_state.is_in_board(x, y) {
            return false;
        }

        // そこが目的地、かつそこに既にたこ焼きがないときのみ離すべき
        self.board_state.final_board[x as usize][y as usize]
            && !self.board_state.cur_board[x as usize][y as usize]
    }

    /// 1手(1turn)のアクションを作成する
    fn action(&mut self) {
        let p = if self.mode_catch {
            self.find_nearest_takoyaki()
        } else {
            self.find_nearest_vacant()
        };
        let dist = calc_dist(&self.arm_state.root, &p);
        let root_move = if dist == 1 {
            '.'
        } else if self.arm_state.root.x < p.x {
            'D'
        } else if self.arm_state.root.x > p.x {
            'U'
        } else if self.arm_state.root.y > p.y {
            'L'
        } else if self.arm_state.root.y < p.y {
            'R'
        } else {
            self.get_legal_root_move()[0]
        };
        assert!(['.', 'D', 'U', 'L', 'R'].contains(&root_move));
        let direction_dif = HashMap::from([
            ('U', (-1, 0)),
            ('D', (1, 0)),
            ('L', (0, -1)),
            ('R', (0, 1)),
            ('.', (0, 0)),
        ]);
        let dx = direction_dif[&root_move].0;
        let dy = direction_dif[&root_move].1;
        self.arm_state.root.x += dx;
        self.arm_state.root.y += dy;

        let mut operation = Operation {
            arm_move: root_move,
            node_rotate: vec!['.'; self.arm_state.num_node],
            node_interact: vec!['.'; self.arm_state.num_node],
        };

        let direction_clockwise = HashMap::from([('U', 'R'), ('R', 'D'), ('D', 'L'), ('L', 'U')]);
        let direction_anti_clockwise =
            HashMap::from([('U', 'L'), ('L', 'D'), ('D', 'R'), ('R', 'U')]);
        let direction_opposite = HashMap::from([('U', 'D'), ('D', 'U'), ('L', 'R'), ('R', 'L')]);

        for u in 1..self.arm_state.num_node {
            let direction = self.arm_state.node_direction[u];
            let have_takoyaki = self.arm_state.node_have_takoyaki[u];

            let leaf_x = self.arm_state.root.x + direction_dif[&direction].0;
            let leaf_y = self.arm_state.root.y + direction_dif[&direction].1;

            let leaf_clock_x =
                self.arm_state.root.x + direction_dif[&direction_clockwise[&direction]].0;
            let leaf_clock_y =
                self.arm_state.root.y + direction_dif[&direction_clockwise[&direction]].1;

            let leaf_anti_x =
                self.arm_state.root.x + direction_dif[&direction_anti_clockwise[&direction]].0;
            let leaf_anti_y =
                self.arm_state.root.y + direction_dif[&direction_anti_clockwise[&direction]].1;

            let leaf_opp_x =
                self.arm_state.root.x + direction_dif[&direction_opposite[&direction]].0;
            let leaf_opp_y =
                self.arm_state.root.y + direction_dif[&direction_opposite[&direction]].1;

            if have_takoyaki {
                if self.should_release_takoyaki(leaf_x, leaf_y) {
                    // そのままたこ焼きを置く
                    operation.node_interact[u] = 'P';
                    self.arrived_count += 1;
                    self.board_state.cur_board[leaf_x as usize][leaf_y as usize] = true;
                    self.arm_state.node_have_takoyaki[u] = false;
                } else if self.should_release_takoyaki(leaf_clock_x, leaf_clock_y) {
                    // 時計回りに回転してたこ焼きを置く
                    operation.node_rotate[u] = 'R';
                    operation.node_interact[u] = 'P';
                    self.arm_state.node_direction[u] = direction_clockwise[&direction];
                    self.arrived_count += 1;
                    self.board_state.cur_board[leaf_clock_x as usize][leaf_clock_y as usize] = true;
                    self.arm_state.node_have_takoyaki[u] = false;
                } else if self.should_release_takoyaki(leaf_anti_x, leaf_anti_y) {
                    // 半時計回りに回転してたこ焼きを置く
                    operation.node_rotate[u] = 'L';
                    operation.node_interact[u] = 'P';
                    self.arm_state.node_direction[u] = direction_anti_clockwise[&direction];
                    self.arrived_count += 1;
                    self.board_state.cur_board[leaf_anti_x as usize][leaf_anti_y as usize] = true;
                    self.arm_state.node_have_takoyaki[u] = false;
                } else if self.should_release_takoyaki(leaf_opp_x, leaf_opp_y) {
                    // 180度向こうなのでまず時計回りに１回回しておく
                    operation.node_rotate[u] = 'R';
                    self.arm_state.node_direction[u] = direction_clockwise[&direction];
                } else {
                    // // 0.5 の確率で適当に時計周りに回す
                    // let do_balance = rand::random::<bool>();
                    // if do_balance {
                    //     operation.node_rotate[u] = 'R';
                    //     self.arm_state.node_direction[u] = direction_clockwise[&direction];
                    // }
                }
            } else {
                if self.should_catch_takoyaki(leaf_x, leaf_y) {
                    // たこ焼きを拾う
                    operation.node_interact[u] = 'P';
                    self.board_state.cur_board[leaf_x as usize][leaf_y as usize] = false;
                    self.arm_state.node_have_takoyaki[u] = true;
                } else if self.should_catch_takoyaki(leaf_clock_x, leaf_clock_y) {
                    operation.node_rotate[u] = 'R';
                    operation.node_interact[u] = 'P';
                    self.arm_state.node_direction[u] = direction_clockwise[&direction];

                    self.board_state.cur_board[leaf_clock_x as usize][leaf_clock_y as usize] =
                        false;
                    self.arm_state.node_have_takoyaki[u] = true;
                } else if self.should_catch_takoyaki(leaf_anti_x, leaf_anti_y) {
                    operation.node_rotate[u] = 'L';
                    operation.node_interact[u] = 'P';
                    self.arm_state.node_direction[u] = direction_anti_clockwise[&direction];

                    self.board_state.cur_board[leaf_anti_x as usize][leaf_anti_y as usize] = false;
                    self.arm_state.node_have_takoyaki[u] = true;
                } else if self.should_catch_takoyaki(leaf_opp_x, leaf_opp_y) {
                    operation.node_rotate[u] = 'R';
                    self.arm_state.node_direction[u] = direction_clockwise[&direction];
                } else {
                    // // 0.5 の確率で適当に時計周りに回す
                    // let do_balance = rand::random::<bool>();
                    // if do_balance {
                    //     operation.node_rotate[u] = 'R';
                    //     self.arm_state.node_direction[u] = direction_clockwise[&direction];
                    // }
                }
            }
        }

        if self.arrived_count + self.arm_state.count_have_takoyaki() == self.board_state.m
            || self.arm_state.is_full()
        {
            eprintln!("Switch to release mode.");
            self.mode_catch = false;
        }

        if self.arm_state.is_empty() {
            self.mode_catch = true;
        }

        eprintln!("{}", operation);
        self.operations.push(operation)
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

struct StarArmState {
    root: Point,
    init_root: Point,
    num_node: usize,
    node_direction: Vec<char>,
    node_have_takoyaki: Vec<bool>,
}

impl StarArmState {
    fn count_have_takoyaki(&self) -> usize {
        self.node_have_takoyaki.iter().filter(|&x| *x).count()
    }
    fn is_empty(&self) -> bool {
        self.count_have_takoyaki() == 0
    }
    fn is_full(&self) -> bool {
        self.count_have_takoyaki() == self.num_node - 1
    }
    fn print_init_arm_state(&self) {
        println!("{}", self.num_node);
        for _ in 1..self.num_node {
            println!("{} {}", 0, 1);
        }
        println!("{} {}", self.init_root.x, self.init_root.y);
    }
}

/// pとqが盤面上で隣接しているか、すなわちpの上下左右にqが存在するか判定
fn is_close(p: &Point, q: &Point) -> bool {
    let is_same_col = (p.x - q.x).abs() == 1 && p.y == q.y;
    let is_same_row = p.x == q.x && (p.y - q.y).abs() == 1;
    is_same_row || is_same_col
}

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
    let mut init_board = vec![vec![false; n]; n];
    for i in 0..n {
        input! { si: Chars };
        assert_eq!(si.len(), n);
        for j in 0..n {
            if si[j] == '1' {
                init_board[i][j] = true;
            }
        }
    }

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

    let board_state = BoardState {
        n,
        m,
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
            if board_state.cur_board[i][j] && board_state.final_board[i][j] {
                arrived_count += 1;
            }
        }
    }

    arrived_count
}

fn calc_dist(p: &Point, q: &Point) -> i32 {
    (p.x - q.x).abs() + (p.y - q.y).abs()
}

fn output_answer(game_state: &GameState) {
    // Armの出力
    game_state.arm_state.print_init_arm_state();

    // 操作列の出力
    assert!(game_state.operations.len() <= MAX_OPERATION_TURN);
    for operation in &game_state.operations {
        println!("{}", operation);
    }
}

fn main() {
    let (_n, _m, v, board_state) = input_parser();
    let arrived_count = count_arrived_takoyaki(&board_state);

    let mut game_state = GameState {
        operations: vec![],
        arm_state: StarArmState {
            root: Point { x: 0, y: 0 }, // とりあえず(0, 0)から始める
            init_root: Point { x: 0, y: 0 },
            num_node: v,
            node_direction: vec!['R'; v],
            node_have_takoyaki: vec![false; v],
        },
        board_state,
        arrived_count,
        mode_catch: true,
    };

    while !game_state.is_finished() {
        game_state.action();
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
            cur_board: vec![
                vec![true, false, false, true],
                vec![false, false, true, false],
                vec![false, false, false, false],
                vec![false, true, false, false],
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

    #[test]
    fn test_is_close() {
        let p = Point { x: 3, y: 2 };
        let q = Point { x: 3, y: 1 };
        assert!(is_close(&p, &q));

        let p = Point { x: 2, y: 0 };
        let q = Point { x: 3, y: 0 };
        assert!(is_close(&p, &q));

        let p = Point { x: 3, y: 2 };
        let q = Point { x: 2, y: 1 };
        assert!(!is_close(&p, &q));
    }
}

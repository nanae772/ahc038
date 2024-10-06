# 問題概要
https://atcoder.jp/contests/ahc038/tasks/ahc038_a

- N * N の盤面がある(15 <= N <= 30)
- 移動させたいたこ焼きがM個ある(N^2 / 10 <= M <= N^2 / 2)
- 移動させるには頂点数V以下のロボットアームを作成しなければならない(5 <= V <= 15)
- ロボットアームの構造は木構造
- ロボットアームの各辺の長さは1以上N未満

各ターンに出来る操作が以下の３つ
1. ロボットアームを上下左右に１マス動かす（動かさなくてもよい）
2. 根以外の各頂点について、その下の部分木を半時計または時計周りに90度回転させる（回転させなくてもよい）
3. 各指先(葉)について、たこ焼きを離すor掴む（既にたこ焼きが置かれている箇所にたこ焼き置けないし、盤外に置くこともできない）

一つの指先で複数のたこ焼きを掴むこともできない

操作は最大で10^5ターン行える

## スコアの計算
操作ターン数をK、終了後に目的地に存在するたこ焼きの数をM'とする
j
- M' = M のとき（全てのたこ焼きが目的地に到達できた場合）、K
- M' < M のとき、10^5 + 1000 * (M - M')

つまり、操作回数いくらかかっても全てのたこ焼きを目的地に置けた方が良い
置けなかったときのペナルティが結構重いかな…
90%のケースで操作回数が切り詰められるけど10%のケースで全部目的地に置けないアルゴリズムより、
手数かかってもいいので100%のケースでちゃんと全部たこ焼きを置けるアルゴリズムのほうが良い
おそらく

# 考察
質問タブにあったんだけど、根しかない木でもいいんだ…
じゃあそれをまず自明解にできるね、自明解というか貪欲解

## ノード数１の貪欲解
ノード数１、つまり根しかない木をロボットアームとして作成する
操作のアルゴリズムとしては
1. 現在の根から最も近くて目標位置に置かれていないたこ焼きを探す
2. それを拾いにいく
3. 拾った地点から最も近い目標位置を見つけて、そこまでいって取ったたこ焼きを配置する
っていう感じかな、シンプルに

まあもっと賢い貪欲解もあるとは思うけど、とりあえずで出す解としてはそれでいいんじゃないか
うーん、多分


## ノード数複数の貪欲解
とりあえず、関節がいっぱいある奴は難しいから
根から直接長さ1のノードがV-1個ついてるやつ、いわゆるスターグラフ（ウニグラフ）を
考えて特に回転がどうこうとか考えずに積載量がV-1個あるロボットが
V-1個たくわえてから各目標地点に置くっていうのを繰り返す、みたいなやつを
実装するといいかなー

# TODO
- input_parserを実装する
- 答えを出力する関数を作る
- 生成した答えがvalidであるかどうかをチェックする関数を作る
- このゲームの状態をどういう構造体で持たせればいいのか考える
  - 盤面の状態（たこ焼きがどのマスにいるのか）
  - ロボットアームの各ノードがどのマスにいるか
  - ロボットアームの各頂点が、親に対してどの向きでついているか
  - ロボットアームの葉がたこ焼きを持っているかいないか
- 各頂点について、その頂点の部分木を時計（半時計）周りに90度回転させたときに、
部分木の各頂点がどの座標に配置されるかを求めるアルゴリズムを考える
  - 木だから再帰的に処理すればそこまで難しくないか？多分

最初からいろいろ難しいことやろうとすると全然進まないから、まずは根のみのやつの
シンプルな貪欲解書くかー

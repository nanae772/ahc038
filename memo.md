頂点0(root)から長さ1の辺が(v-1)個生えている木による解法を作る

どう実装するか？という問題
頂点1,..,v-1がrootに対して現在どの方向を向いているかという状態をvecでもたせるのが
いいかもしれない(移動方向のUDLRと混同しないようにNSWEがいいかな？)
いや一緒のほうがいいかもしれんな、なんか実装上一緒のほうが楽かもしれない
一緒にするか
長さvのVec<char>かな？で、0はダミーで'.'とか入れといて初期値はみんなR
armstateとしては

StarArmState
- root_position: Point
- init_root_position: Point , 最後の出力のためだけだけど一応持っておこう
- num_node: i32
- node_direction: Vec<char>
- node_have_takoyaki: Vec<bool>
- direction_to_node: Hashmap(char -> vec<usize>), 指定した方向にあるノード番号のリスト
これ作ると回転するたびにこれも更新かけなきゃいけないからちょっと面倒かな・・・？
というかVってたかだか15だからいらないかな・・・
- empty_leaf: i32 , たこ焼きを持っていない葉の個数、最大v-1、積載量に余裕があるかO(1)でチェックするために
これもいらないかな、いうて15だもんな

StarArmStateのメソッドとして
- count_leaf_have_takoyaki: たこ焼きを持っている葉の数をカウント
- is_full: たこ焼きを掴める葉があるかどうかチェック
- is_empty: 
- interact_point(p: &Point) -> Vec<Operation>: 点pにinteractするための操作列を作る
- balance: 偏りがあるとあんまり良くなさそうだから、雑に確率で枝を回す仕組みを作る
  - これちょっとどうしようかな…意外と面倒かもしれない…
  - 長さvのVec<char>返せばいいか、rootは動かさないように

GameState
- startArmのルートの上下左右に以下の条件に合致する座標があるかどうか、存在するならその地点のうち１つだけを返す
  - 積載量に余裕があって、目的地に到達していないたこ焼きが存在する
  - 積載量が１以上で、たこ焼きが置かれていない目的地が存在する

アルゴリズムとしては
- 積載量満杯になるまで現在地から最短距離にあるたこ焼きを拾う
  - ただし、持ってるたこ焼きがあって道中で寄り道せずにおけるたこ焼きがあればおいてしまう
- 満杯になったら、今度は現在地から最短距離にある目的地にたこ焼きを置く
  - ただし、積載量に余裕があって道中で寄り道せずに取れるたこ焼きがあれば取る
- 積載量0になったらまた最初に戻る、以下繰り返し

ちょっと設計というか操作列の作り方変えるか
今まではシンプルな１点木だったから一気にガッって作ってたけど
１手ずつ作るほうが操作の柔軟性が高いかもしれない

やり方としては
1. modeパラメーターによって分岐
  1. mode=catchなら、最も近く目的地に到達していないたこ焼きの地点を探す
  2. mode=releaseなら、最も近くたこ焼きが置かれていない目的地の地点を探す
  3. それをpとする
2. rootとpの位置を比較
  1. rootとpの距離が1、つまりrootの上下左右１マスにpがあるなら、動かない
  1. root.x < p.x なら下に動く
  2. root.x > p.x なら上に動く
  3. root.y < p.y なら右に動く
  4. root.y > p.y なら左に動く
  5. root = p、同じ地点にあるなら合法手のうちのどちらかに動く（rootでは取れないので）
    - rootは盤面外にはみ出してはいけないのでそのような動きは除外
3. それぞれの葉について、以下を実行
  1. その葉がたこ焼きを持っていない場合
    1. 現在地にたこ焼きがあるならそれをキャッチ
    2. 現在地から時計（半時計）に90度回したところにたこ焼きがあるなら回してキャッチ
    3. 現在地の対極にたこ焼きがあるならとりあえず90度回す
    4. いずれにも当てはまらなかったら、バランス実行(確率で回す)
  2. その葉がたこ焼きを持っている場合
    1. 現在地が目的地ならそこにリリース
    2. なんかあとは同じ感じ 

これやるならGameStateにmodeパラメータ持たせるか
シンプルにmode_catch=true, falseがいいかな？

センスねえなあ〜、まじで…
たこ焼き置く判定、拾う判定が甘い
置くときは置き先に既にたこ焼きが置かれてるかどうかもチェックしないといけない
拾うときはそこが目的地でないことを確認しなければならない

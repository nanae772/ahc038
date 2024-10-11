import subprocess
import os
import sys

MAX_OPERATION_TURN = 10**5


def parse_input(infile):
    n, m, v = map(int, infile.readline().split())

    cur_board = [[False] * n for _ in range(n)]
    for i in range(n):
        s = infile.readline()
        for j in range(n):
            if s[j] == "1":
                cur_board[i][j] = True

    final_board = [[False] * n for _ in range(n)]
    for i in range(n):
        t = infile.readline()
        for j in range(n):
            if t[j] == "1":
                final_board[i][j] = True

    return (n, m, v, cur_board, final_board)


def print_board(board):
    for row in board:
        s = ""
        for x in row:
            s += "1" if x else "0"
        print(s)


def check(n, m, v, cur_board, final_board, outfile):
    outfile.readline()
    x, y = map(int, outfile.readline().split())
    assert 0 <= x <= n - 1
    assert 0 <= y <= n - 1

    allow_direction = {
        "L": (0, -1),
        "R": (0, 1),
        "U": (-1, 0),
        "D": (1, 0),
        ".": (0, 0),
    }
    allow_interaction = {".", "P"}
    operation_count = 0
    has_takoyaki = False

    for line in outfile:
        operation_count += 1
        direction = line[0]
        if direction not in allow_direction:
            raise IllegalDirectionError(f"Illegal Direction: {direction}")
        dx, dy = allow_direction[direction]
        x += dx
        y += dy
        if x < 0 or x >= n or y < 0 or y >= n:
            raise OutOfBoardError(f"out of board(turn {operation_count}): ({x}, {y})")

        interaction = line[1]
        if interaction not in allow_interaction:
            raise IllegalInteractionError(
                f"Illegal Interaction:(turn {operation_count}) {interaction}"
            )
        if interaction == "P":
            # たこ焼きが存在しないところからたこ焼きを取ろうとした
            if not cur_board[x][y] and not has_takoyaki:
                print_board(cur_board)
                raise NotFoundTakoyakiError(
                    f"(turn {operation_count})Not found takoyaki at ({x}, {y})"
                )
            # たこ焼きが存在しているところにたこ焼きを置こうとした
            if cur_board[x][y] and has_takoyaki:
                raise DuplicationTakoyakiError(
                    f"(turn {operation_count})Takoyaki already exists at ({x}, {y})"
                )
            if has_takoyaki:
                cur_board[x][y] = True
                has_takoyaki = False
            else:
                cur_board[x][y] = False
                has_takoyaki = True

    if operation_count > MAX_OPERATION_TURN:
        raise ExceedMaxOperationTurnError(f"Operation count is over: {operation_count}")


class DuplicationTakoyakiError(Exception):
    pass


class ExceedMaxOperationTurnError(Exception):
    pass


class NotFoundTakoyakiError(Exception):
    pass


class IllegalInteractionError(Exception):
    pass


class OutOfBoardError(Exception):
    pass


class IllegalDirectionError(Exception):
    pass


class IllegalTakoyakiInteractionError(Exception):
    pass


def main():
    for input_file in sorted(os.listdir("inputs")):
        input_path = os.path.join("inputs", input_file)
        output_file = os.path.join("outputs", input_file)

        print(f"Checking {input_file}...")

        try:
            with open(input_path, "r") as infile, open(output_file, "r") as outfile:
                n, m, v, cur_board, final_board = parse_input(infile)
                check(n, m, v, cur_board, final_board, outfile)
        except Exception as e:
            print(f"An unexpected error occurred: {e}")
            sys.exit(1)

    print("All files processed successfully.")


if __name__ == "__main__":
    main()

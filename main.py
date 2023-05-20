from board import Board, draw_board_black, draw_board_white
import os


def clear():
    os.system("cls || clear")


def newGame():
    clear()
    player1 = input("Enter name for White player: ")
    player2 = input("Enter name for Black player: ")
    turn = 0

    x = Board.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    while input != "q":
        match turn:
            case 0:
                # White
                draw_board_white(x)
                move = input(f"{player1}, It's your turn!\n")
                # Add logic for chess inputs
                move_arr = move.split(" ")
                print("Move: ", move_arr)
                # input: 0 1 0 3
                x.move(
                    (int(move_arr[0]), int(move_arr[1])),
                    (int(move_arr[2]), int(move_arr[3])),
                )
            case 1:
                # Black
                draw_board_black(x)
                move = input(f"{player2}, It's your turn!\n")
                move[0]
        turn = not turn


def loadGame():
    fen = input("Input the fen string of the game you would like to load ->")


def main():
    clear()
    print("Welcome to CLI-Chess -> \n")
    ans = input("1. Play New Game\n2. Load Game from FEN\n")
    if ans == "1":
        newGame()
    elif ans == "2":
        loadGame()


main()

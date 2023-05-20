from board import Board, draw_board_black, draw_board_white

x = Board.from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
print(x.piece(0, 1).type)
print(x.get_valid_moves(0, 1))
x.piece(0, 1).move_to(0, 3)
draw_board_white(x)

from __future__ import annotations

from dataclasses import dataclass, field

from moves import (
    Position,
    get_valid_moves_rook,
    get_valid_moves_pawn,
    get_valid_moves_bishop,
    get_valid_moves_knight,
    get_valid_moves_queen,
    get_possible_moves_king,
)
from pieces import Colour, Piece, PieceType

Grid = {}
# dict[Position, Piece]


def empty_board() -> Grid:
    grid: Grid = {}
    for x in range(8):
        for y in range(8):
            grid[(x, y)] = Piece(x, y)
    return grid


@dataclass
class Board:
    pieces: Grid = field(default_factory=empty_board)

    @staticmethod  # https://www.chess.com/terms/fen-chess fen strings start from top left (0,7)
    def from_fen(fen: str) -> Board:
        board = Board()
        fenlist = fen.split("/")

        for indy, y in enumerate(fenlist):
            extra = 0
            for indx, x in enumerate(y):
                if x.isnumeric():
                    for i in range(int(x)):
                        if i > 0:
                            extra += 1
                        # default Piece is an empty square
                        board.place(Piece(indx + extra, 7 - indy))
                else:
                    # from_fen places an actual Piece
                    board.place(Piece.from_fen(indx + extra, 7 - indy, x))
        return board

    def place(self, piece: Piece) -> None:
        self.pieces[(piece.x, piece.y)] = piece

    def piece(self, x: int, y: int) -> Piece:
        return self.pieces[(x, y)]

    def piece_type(self, x: int, y: int) -> PieceType:
        return self.piece(x, y).type

    def empty(self, x: int, y: int) -> bool:
        return self.piece(x, y).type == PieceType.EMPTY

    def find_king(self, colour: Colour) -> Piece:
        for piece in self.pieces.values():
            if piece.type == PieceType.KING and colour == piece.colour:
                return piece

    def get_valid_moves(self, x: int, y: int) -> list[Position]:
        return MOVE_LISTS[self.piece_type(x, y)]


MOVE_LISTS = {
    PieceType.PAWN: get_valid_moves_pawn,
    PieceType.BISHOP: get_valid_moves_bishop,
    PieceType.KNIGHT: get_valid_moves_knight,
    PieceType.ROOK: get_valid_moves_rook,
    PieceType.QUEEN: get_valid_moves_queen,
    PieceType.KING: get_possible_moves_king,
}


def draw_board_white(board: Board) -> None:
    for y in range(7, -1, -1):
        for x in range(8):
            print(board.piece(x, y), end=" ")
        print("\t")


def draw_board_black(board: Board) -> None:
    for y in range(8):
        for x in range(8):
            print(board.piece(x, y), end=" ")
        print("\t")

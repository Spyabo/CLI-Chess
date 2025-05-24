from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class PieceType(Enum):
    EMPTY = "empty"
    PAWN = "pawn"
    ROOK = "rook"
    BISHOP = "bishop"
    QUEEN = "queen"
    KNIGHT = "knight"
    KING = "king"


class Colour(Enum):
    NONE = -1
    WHITE = 0
    BLACK = 1


PIECE_STR: dict[PieceType, tuple[str, str]] = {
    PieceType.EMPTY: ("·", "·"),
    PieceType.PAWN: ("♟", "♙"),
    PieceType.ROOK: ("♜", "♖"),
    PieceType.BISHOP: ("♝", "♗"),
    PieceType.QUEEN: ("♛", "♕"),
    PieceType.KING: ("♚", "♔"),
    PieceType.KNIGHT: ("♞", "♘"),
}


FEN_MAP: dict[str, PieceType] = {
    "p": PieceType.PAWN,
    "r": PieceType.ROOK,
    "b": PieceType.BISHOP,
    "q": PieceType.QUEEN,
    "k": PieceType.KING,
    "n": PieceType.KNIGHT,
}


@dataclass
class Piece:
    x: int
    y: int
    colour: Colour = Colour.NONE
    type: PieceType = PieceType.EMPTY
    has_moved: bool = False
    moves_made: int = 0
    last_moved: int = 0

    @staticmethod
    def from_fen(x: int, y: int, fen: str) -> Piece:
        colour = Colour.WHITE if fen.isupper() else Colour.BLACK
        return Piece(x, y, colour, type=FEN_MAP[fen.lower()])

    def move_to(self, x: int, y: int) -> None:
        self.x, self.y = x, y
        self.has_moved = True
        self.moves_made += 1

    def promote_to_queen(self) -> None:
        self.type = PieceType.QUEEN

    @property
    def image(self) -> str:
        return f"pieces/{self.type}{self.colour.value.value}.png"

    def __str__(self):
        return PIECE_STR[self.type][self.colour.value]

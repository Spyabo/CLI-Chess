from typing import Protocol

from pieces import Piece

Position = tuple([int, int])


class Board_squares(Protocol):
    def empty(self, x: int, y: int) -> bool:
        """Whether the square (x, y) is empty."""

    def piece(self, x: int, y: int) -> Piece:
        """Returns the piece at position (x, y)."""


def get_valid_moves_pawn(square: Board_squares, x: int, y: int) -> list[Position]:
    valid_moves: list[Position] = []
    #Moves for white pawns
    if square.piece(x, y).colour(0):
        #Pawns can move 2 squares on 1st move
        if y == 1 and square.empty(x, y + 2):
            valid_moves.append((x, y + 2))
        #checking top left and right captures
        if 0 < x > 7:
            for i in range(-1,1,2):
                if square.piece(x + i, y + 1).colour != 1:
                    valid_moves.append((x + i, y + 1))
        #edge pawn so doesn't check outside of the board
        elif x == 0:
            if square.piece(x + 1, y + 1).colour != 1:
                    valid_moves.append((x + 1, y + 1))
        elif x == 7:
            if square.piece(x - 1, y + 1).colour != 1:
                    valid_moves.append((x - 1, y + 1))
        #check the square infont of current pawn
        if square.empty(x, y + 1):
            valid_moves.append(x, y + 1)
        return valid_moves

    #Moves for black pawns
    if square.piece(x, y).colour(1):
        if y == 6 and square.empty(x, y - 2):
            valid_moves.append((x, y - 2))
        #checking top left and right captures
        if 0 < x > 7:
            for i in range(-1,1,2):
                if square.piece(x + i, y - 1).colour != 0:
                    valid_moves.append((x + i, y - 1))
        #edge pawn so doesn't check outside of the board
        elif x == 0:
            if square.piece(x + 1, y - 1).colour != 0:
                    valid_moves.append((x + 1, y - 1))
        elif x == 7:
            if square.piece(x - 1, y - 1).colour != 0:
                    valid_moves.append((x - 1, y - 1))
        #check the square infont of current pawn
        if square.empty(x, y - 1):
            valid_moves.append(x, y - 1)
        return valid_moves
    
    
        

def get_valid_moves_bishop(square: Board_squares, x: int, y: int) -> list[Position]:
    valid_moves: list[Position] = []
    
    

def get_valid_moves_rook(square: Board_squares, x: int, y: int) -> list[Position]:
    valid_moves: list[Position] = []
    #Check squares to the right of the piece
    for i in range(1, 8 - x):
        if square.empty(x + i, y):
            valid_moves.append((x + i, y))
        elif square.piece(x + i, y).colour != square.piece(x, y).colour:
            valid_moves.append((x + i, y))
            break
        else:
            break
    #Check squares to the left of the piece
    for i in range(1, x + 1):
        if square.empty(x - i, y):
            valid_moves.append((x - i, y))
        elif square.piece(x, y).colour != square.piece(y, x - i).colour:
            valid_moves.append((x - i, y))
            break
        else:
            break
    #Check squares above the piece        
    for i in range(1, 8 - y):
        if square.empty(x, y + i):
            valid_moves.append((x, y + i))
        elif square.piece(x, y).colour != square(x, y + i).colour:
            valid_moves.append((x, y + i))
            break
        else:
            break
    #Check sqaures below the piece
    for i in range(1, y + 1):
        if square.empty(x, y - i):
            valid_moves.append((x, y - i))
        elif square.piece(x, y).colour != square.piece(x, y - i).colour:
            valid_moves.append((x, y - i))
            break
        else:
            break
    return valid_moves
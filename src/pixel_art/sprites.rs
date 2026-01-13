use super::Pixel;
use Pixel::*;

/// Sprite dimensions - 5 pixels wide, 8 pixels tall
/// With half-blocks, this renders as 5 chars wide x 4 chars tall
pub const SPRITE_WIDTH: usize = 5;
pub const SPRITE_HEIGHT: usize = 8;

/// A piece sprite - width x height pixels
pub type PieceSprite = [[Pixel; SPRITE_WIDTH]; SPRITE_HEIGHT];

/// Container for all piece sprites
pub struct PieceSprites {
    pub pawn: PieceSprite,
    pub knight: PieceSprite,
    pub bishop: PieceSprite,
    pub rook: PieceSprite,
    pub queen: PieceSprite,
    pub king: PieceSprite,
}

impl Default for PieceSprites {
    fn default() -> Self {
        Self {
            pawn: PAWN_SPRITE,
            knight: KNIGHT_SPRITE,
            bishop: BISHOP_SPRITE,
            rook: ROOK_SPRITE,
            queen: QUEEN_SPRITE,
            king: KING_SPRITE,
        }
    }
}

/// Pawn sprite (5x8) - compact pawn shape
pub const PAWN_SPRITE: PieceSprite = [
    [Transparent, Outline,     Outline,     Outline,     Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Transparent, Transparent, Outline,     Transparent, Transparent],
    [Transparent, Transparent, Outline,     Transparent, Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Outline,     Outline,     Outline,     Outline    ],
];

/// Knight sprite (5x8) - horse head profile
pub const KNIGHT_SPRITE: PieceSprite = [
    [Transparent, Outline,     Outline,     Transparent, Transparent],
    [Outline,     Primary,     Primary,     Outline,     Transparent],
    [Outline,     Outline,     Primary,     Primary,     Outline    ],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Outline,     Outline,     Outline,     Outline    ],
];

/// Bishop sprite (5x8) - mitre with slit
pub const BISHOP_SPRITE: PieceSprite = [
    [Transparent, Transparent, Outline,     Transparent, Transparent],
    [Transparent, Outline,     Outline,     Outline,     Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Transparent, Transparent, Outline,     Transparent, Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Outline,     Outline,     Outline,     Outline    ],
];

/// Rook sprite (5x8) - castle tower with battlements
pub const ROOK_SPRITE: PieceSprite = [
    [Outline,     Transparent, Outline,     Transparent, Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Outline,     Outline,     Outline,     Outline    ],
];

/// Queen sprite (5x8) - crown with points
pub const QUEEN_SPRITE: PieceSprite = [
    [Accent,      Transparent, Accent,      Transparent, Accent     ],
    [Outline,     Accent,      Outline,     Accent,      Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Outline,     Outline,     Outline,     Outline    ],
];

/// King sprite (5x8) - crown with cross
pub const KING_SPRITE: PieceSprite = [
    [Transparent, Transparent, Accent,      Transparent, Transparent],
    [Transparent, Accent,      Accent,      Accent,      Transparent],
    [Transparent, Outline,     Primary,     Outline,     Transparent],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Primary,     Outline,     Primary,     Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Primary,     Primary,     Primary,     Outline    ],
    [Outline,     Outline,     Outline,     Outline,     Outline    ],
];

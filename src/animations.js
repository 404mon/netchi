export const animationsConfig = {
  "Egg": {
    "idle": { frames: [24, 25], speed_ms: 1000 },
    "hatch": { frames: [26, 27], speed_ms: 300, loop: false }
  },
  "Baby": {
    "idle": { frames: [16, 17], speed_ms: 600 },
    "sleeping": { frames: [18, 19], speed_ms: 1200 },
    "hungry": { frames: [20, 21], speed_ms: 500 }, // Corrisponde al "crying"
    "eating": { frames: [22, 23, 22, 23, 22, 23, 22, 23, 22, 23], speed_ms: 300, loop: false } 
  },
  "Adult": {
    "idle": { frames: [0, 1], speed_ms: 800 },
    "hungry": { frames: [2, 3], speed_ms: 600 },
    "tired": { frames: [4, 5], speed_ms: 1000 },
    "sleeping": { frames: [6, 7], speed_ms: 1200 },
    "eating": { frames: [8, 9, 8, 9, 8, 9, 8, 9, 8, 9], speed_ms: 300, loop: false },
    "dying": { frames: [10, 11], speed_ms: 1500 },
    "surprised": { frames: [12, 13, 12, 13, 12, 13], speed_ms: 400, loop: false },
    "fat": { frames: [14, 15], speed_ms: 600 }
  }
};
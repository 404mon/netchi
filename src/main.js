import { animationsConfig } from './animations.js';
import './styles.css';

// --- AUDIO ENGINE INIT ---
const sfx = {
  beep: new Audio('/beep.mp3'),
  scan: new Audio('/scan.mp3'),
  hatch: new Audio('/hatch.mp3')
};
// Elegant volume leveling
sfx.beep.volume = 0.15;
sfx.scan.volume = 0.25;
sfx.hatch.volume = 0.3;

const petPlaceholder = document.querySelector('.netchi-sprite');

let currentAnimName = ''; 
let currentFrameIndex = 0;
let animationTimer = null;
let currentStage = "Egg"; 
let isTransitioning = false; 

// --- ANIMATION ENGINE ---
function playAnimation(animName, stage, onComplete = null) {
  if (currentAnimName === animName && animationTimer !== null && !onComplete) return;
  
  currentAnimName = animName;
  currentFrameIndex = 0;
  if (animationTimer) clearInterval(animationTimer);

  const animData = animationsConfig[stage]?.[animName];
  if (!animData) {
    console.warn(`[SYS] ANIMATION_NOT_FOUND: ${animName} for ${stage}`);
    return;
  }

  const updateFrame = () => {
    const frameNum = animData.frames[currentFrameIndex];
    petPlaceholder.style.backgroundPosition = `-${frameNum * 64}px 0`;
    
    if (currentFrameIndex === animData.frames.length - 1 && animData.loop === false) {
      clearInterval(animationTimer);
      animationTimer = null;
      if (onComplete) onComplete();
    } else {
      currentFrameIndex = (currentFrameIndex + 1) % animData.frames.length;
    }
  };

  updateFrame();
  animationTimer = setInterval(updateFrame, animData.speed_ms);
}

// Initial Boot (Egg)
playAnimation('idle', "Egg");

// --- RUST BACKEND LISTENERS ---
if (window.__TAURI__) {
  const { listen } = window.__TAURI__.event;

  // Audio trigger synchronized with UNIX logs
  listen('sys-log', (event) => {
    const msg = event.payload;
    if (msg.includes("ACTIVE_RECON_INITIATED") || msg.includes("ACTIVE_SCAN_RUNNING") || msg.includes("FORCING_ACTIVE_SCAN")) {
        sfx.scan.play().catch(e => console.log("[SYS] AUDIO_PLAY_BLOCKED_BY_BROWSER"));
    }
  });

  // State & Animation Sync
  listen('state-update', (event) => {
    const state = event.payload; 

    // SKIN MANAGEMENT
    const skinMap = {
      "ghost": "spritesheet.png",
      "duck": "spritesheet1.png",
      "beagle": "spritesheet3.png"
    };
    const skinFileName = skinMap[state.skin] || "spritesheet.png";
    const newBg = `url('/${skinFileName}')`; 
    
    if (petPlaceholder.style.backgroundImage !== newBg) {
      petPlaceholder.style.backgroundImage = newBg;
    }

    // EVOLUTION HANDLING (Hatch)
    if (currentStage === "Egg" && state.stage === "Baby" && !isTransitioning) {
      isTransitioning = true;
      sfx.hatch.play().catch(e => {}); 
      
      playAnimation('hatch', "Egg", () => {
        isTransitioning = false;
        currentStage = "Baby";
        playAnimation(state.current_action, "Baby");
      });
      return;
    }

    if (isTransitioning) return;
    currentStage = state.stage;

    // Beep on action change (excluding Egg stage)
    if (state.current_action !== currentAnimName) {
        if (currentStage !== "Egg") {
            sfx.beep.currentTime = 0;
            sfx.beep.play().catch(e => {});
        }
        playAnimation(state.current_action, currentStage);
    }
  });

  // Sentinel Mode (Visual alarm)
  listen('intruder-alert', async (event) => {
    const { getCurrentWindow } = window.__TAURI__.window;
    const appWin = getCurrentWindow();
    await appWin.show();
    await appWin.setFocus();

    playAnimation('surprised', currentStage);
  });
}

// --- GLOBAL KEYBINDS ---
document.addEventListener('keydown', async (e) => {
  if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

  // Toggle Dashboard
  if (e.key.toLowerCase() === 'd' && window.__TAURI__) {
    const { Window } = window.__TAURI__.window;
    const dash = await Window.getByLabel('dashboard');
    if (dash) {
      (await dash.isVisible()) ? await dash.hide() : await dash.show();
    } else {
      const newDash = new Window('dashboard', { 
        url: 'dashboard.html', 
        title: 'Netchi',
        decorations: false,
        width: 800,
        height: 500
      });
      await newDash.show();
    }
  }

  // Force Manual Scan
  if (e.key.toLowerCase() === 's' && window.__TAURI__) {
    const { invoke } = window.__TAURI__.core; 
    console.log("[USER] FORCING_MANUAL_SCAN...");
    invoke('trigger_active_scan').catch(err => console.error("[ERR] NMAP_EXECUTION_FAILED:", err));
  }
});

// Dragging override
document.getElementById('pet').addEventListener('mousedown', (e) => {
  if (e.button === 0 && window.__TAURI__) {
    const { getCurrentWindow } = window.__TAURI__.window;
    getCurrentWindow().startDragging();
  }
});

// Disable context menu
document.addEventListener('contextmenu', (e) => e.preventDefault());
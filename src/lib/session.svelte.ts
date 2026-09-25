class Session {
  unlocked = $state(false);
  airlockPassed = $state(false);
}

export const session = new Session();

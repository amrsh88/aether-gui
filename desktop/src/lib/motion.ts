import type { Transition, Variants } from "framer-motion";

/**
 * Shared motion vocabulary.
 *
 * Every animation in the app pulls its timing from here so the whole UI feels
 * like one object rather than a pile of independently-tuned widgets.
 */

/** Snappy spring for anything the user directly presses. */
export const springPress: Transition = {
  type: "spring",
  stiffness: 400,
  damping: 17,
  mass: 0.6,
};

/** Softer spring for layout shifts and pill sliding. */
export const springSoft: Transition = {
  type: "spring",
  stiffness: 260,
  damping: 26,
};

/** Standard ease for opacity/colour crossfades. */
export const easeOut: Transition = {
  duration: 0.28,
  ease: [0.16, 1, 0.3, 1],
};

/** Text that swaps in place (status line, timer label). */
export const swapText: Variants = {
  initial: { opacity: 0, y: 8, filter: "blur(4px)" },
  animate: { opacity: 1, y: 0, filter: "blur(0px)" },
  exit: { opacity: 0, y: -8, filter: "blur(4px)" },
};

/** Whole-page transition, driven by AnimatePresence mode="wait". */
export const swapPage: Variants = {
  initial: { opacity: 0, x: 18 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -18 },
};

/** Parent of a staggered list of cards. */
export const staggerParent: Variants = {
  initial: {},
  animate: {
    transition: { staggerChildren: 0.06, delayChildren: 0.04 },
  },
};

/** Child of `staggerParent`. */
export const staggerChild: Variants = {
  initial: { opacity: 0, y: 12 },
  animate: { opacity: 1, y: 0, transition: easeOut },
};

/** Toast sliding up from the bottom edge. */
export const toastRise: Variants = {
  initial: { opacity: 0, y: 24, scale: 0.96 },
  animate: { opacity: 1, y: 0, scale: 1, transition: springSoft },
  exit: { opacity: 0, y: 12, scale: 0.98, transition: { duration: 0.16 } },
};

/** Accordion body reveal. */
export const collapse: Variants = {
  initial: { height: 0, opacity: 0 },
  animate: { height: "auto", opacity: 1, transition: { duration: 0.26, ease: [0.16, 1, 0.3, 1] } },
  exit: { height: 0, opacity: 0, transition: { duration: 0.2, ease: [0.16, 1, 0.3, 1] } },
};

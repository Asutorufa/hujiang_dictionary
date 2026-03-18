import { useEffect, useState, useRef } from "react";

export function useScrollDirection() {
  const [direction, setDirection] = useState<"up" | "down" | null>(null);
  const lastScrollY = useRef(0);
  const directionRef = useRef<"up" | "down" | null>(null);
  const tickingRef = useRef(false);
  const threshold = 15; // increased threshold to reduce jitter and state updates

  useEffect(() => {
    lastScrollY.current = window.scrollY;

    const updateScrollDirection = () => {
      const scrollY = window.scrollY;
      const diff = scrollY - lastScrollY.current;

      // Ensure nav is always visible at the very top
      if (scrollY <= threshold) {
        if (directionRef.current !== "up") {
          directionRef.current = "up";
          setDirection("up");
        }
        lastScrollY.current = scrollY;
        return;
      }

      // Check if scroll delta exceeded threshold
      if (Math.abs(diff) > threshold) {
        const newDirection = diff > 0 ? "down" : "up";
        if (newDirection !== directionRef.current) {
          directionRef.current = newDirection;
          setDirection(newDirection);
        }
        lastScrollY.current = scrollY;
      }
    };

    const onScroll = () => {
      if (tickingRef.current) return;
      tickingRef.current = true;
      window.requestAnimationFrame(() => {
        updateScrollDirection();
        tickingRef.current = false;
      });
    };

    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return direction;
}

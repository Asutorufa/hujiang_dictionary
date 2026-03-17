import { useEffect, useState, useRef } from "react";

export function useScrollDirection() {
  const [direction, setDirection] = useState<"up" | "down" | null>(null);
  const lastScrollY = useRef(0);
  const threshold = 10; // increase threshold slightly to reduce jitter

  useEffect(() => {
    lastScrollY.current = window.scrollY;

    const updateScrollDirection = () => {
      const scrollY = window.scrollY;
      const diff = scrollY - lastScrollY.current;

      // Ensure nav is always visible at the very top
      if (scrollY <= threshold) {
        if (direction !== "up") setDirection("up");
        lastScrollY.current = scrollY;
        return;
      }

      // Check if scroll delta exceeded threshold
      if (Math.abs(diff) > threshold) {
        const newDirection = diff > 0 ? "down" : "up";
        if (newDirection !== direction) {
          setDirection(newDirection);
        }
        lastScrollY.current = scrollY;
      }
    };

    window.addEventListener("scroll", updateScrollDirection, { passive: true });
    return () => window.removeEventListener("scroll", updateScrollDirection);
  }, [direction]);

  return direction;
}

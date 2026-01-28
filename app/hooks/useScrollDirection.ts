import { useEffect, useState, useRef } from 'react';

export function useScrollDirection() {
  const [scrollDirection, setScrollDirection] = useState<'up' | 'down' | null>(null);
  const lastScrollY = useRef(0);

  useEffect(() => {
    const updateScrollDirection = () => {
      const scrollY = window.scrollY;
      const direction = scrollY > lastScrollY.current ? 'down' : 'up';

      if (direction !== scrollDirection && Math.abs(scrollY - lastScrollY.current) > 5) {
        setScrollDirection(direction);
      }
      lastScrollY.current = scrollY > 0 ? scrollY : 0;
    };

    window.addEventListener('scroll', updateScrollDirection);
    return () => {
      window.removeEventListener('scroll', updateScrollDirection);
    };
  }, [scrollDirection]); // keeping scrollDirection in dep array is needed to compare inside closure unless we use functional update or ref for direction too.
  // Actually, standard implementation often just re-attaches or uses a mutable ref for "blocking" updates.
  // Re-attaching on direction change is infrequent enough (only when you change direction) so it's fine.

  return scrollDirection;
}

import { motion } from "framer-motion";

/**
 * Tiny filled area chart of the last N throughput samples.
 *
 * Drawn as a normalised 0..1 path so it scales to any box without recomputing
 * on resize. The line animates its `pathLength` on mount, then just re-renders
 * as samples arrive.
 */
export function Sparkline({
  values,
  color,
  height = 30,
}: {
  values: number[];
  color: string;
  height?: number;
}) {
  const W = 100;
  const H = 100;

  if (values.length < 2) {
    return <div style={{ height }} className="w-full" />;
  }

  const peak = Math.max(...values, 1);
  const step = W / (values.length - 1);

  const points = values.map((v, i) => {
    const x = i * step;
    const y = H - (v / peak) * H * 0.92 - 4;
    return `${x.toFixed(2)},${y.toFixed(2)}`;
  });

  const line = `M ${points.join(" L ")}`;
  const area = `${line} L ${W},${H} L 0,${H} Z`;
  const gradientId = `spark-${color.replace(/[^a-z0-9]/gi, "")}`;

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      preserveAspectRatio="none"
      style={{ height }}
      className="w-full overflow-visible"
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.28" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <path d={area} fill={`url(#${gradientId})`} />
      <motion.path
        d={line}
        fill="none"
        stroke={color}
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        vectorEffect="non-scaling-stroke"
        initial={{ pathLength: 0 }}
        animate={{ pathLength: 1 }}
        transition={{ duration: 0.5, ease: "easeOut" }}
      />
    </svg>
  );
}

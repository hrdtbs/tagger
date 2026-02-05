import { useState, useEffect, useRef, MouseEvent } from 'react';
import { invoke } from "@tauri-apps/api/core";

interface OverlayProps {
  onClose: () => void;
  onProcess: (selection: {x: number, y: number, w: number, h: number}) => void;
}

export default function Overlay({ onClose, onProcess }: OverlayProps) {
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [selection, setSelection] = useState<{x: number, y: number, w: number, h: number} | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const startPos = useRef<{x: number, y: number} | null>(null);
  const imgRef = useRef<HTMLImageElement>(null);

  useEffect(() => {
    async function capture() {
      try {
        console.log("Invoking capture_screen...");
        const result = await invoke<string>('capture_screen');
        console.log("Screen captured, length:", result.length);
        setImageSrc(result);
      } catch (e) {
        console.error("Failed to capture screen:", e);
        alert("Failed to capture screen: " + e);
        onClose();
      }
    }
    capture();

    const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  const handleMouseDown = (e: MouseEvent) => {
    setIsDragging(true);
    startPos.current = { x: e.clientX, y: e.clientY };
    setSelection({ x: e.clientX, y: e.clientY, w: 0, h: 0 });
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging || !startPos.current) return;
    const currentX = e.clientX;
    const currentY = e.clientY;

    const x = Math.min(startPos.current.x, currentX);
    const y = Math.min(startPos.current.y, currentY);
    const w = Math.abs(currentX - startPos.current.x);
    const h = Math.abs(currentY - startPos.current.y);

    setSelection({ x, y, w, h });
  };

  const handleMouseUp = () => {
    setIsDragging(false);
    if (selection && selection.w > 10 && selection.h > 10 && imgRef.current) {
        // Calculate scaling
        const img = imgRef.current;
        const rect = img.getBoundingClientRect();

        const naturalWidth = img.naturalWidth;
        const naturalHeight = img.naturalHeight;

        if (naturalWidth === 0 || naturalHeight === 0) return;

        const { width, height } = rect;
        const imageRatio = naturalWidth / naturalHeight;
        const containerRatio = width / height;

        let displayWidth, displayHeight, offsetX, offsetY;

        if (containerRatio > imageRatio) {
            displayHeight = height;
            displayWidth = height * imageRatio;
            offsetX = (width - displayWidth) / 2;
            offsetY = 0;
        } else {
            displayWidth = width;
            displayHeight = width / imageRatio;
            offsetX = 0;
            offsetY = (height - displayHeight) / 2;
        }

        const realX = Math.max(0, (selection.x - rect.left - offsetX) * (naturalWidth / displayWidth));
        const realY = Math.max(0, (selection.y - rect.top - offsetY) * (naturalHeight / displayHeight));
        const realW = selection.w * (naturalWidth / displayWidth);
        const realH = selection.h * (naturalHeight / displayHeight);

        onProcess({ x: realX, y: realY, w: realW, h: realH });
    } else {
        setSelection(null);
    }
  };

  if (!imageSrc) return <div className="fixed inset-0 bg-black text-white flex items-center justify-center z-50">Capturing screen...</div>;

  return (
    <div
        className="fixed inset-0 cursor-crosshair z-50 overflow-hidden select-none"
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
    >
      {/* Background Image */}
      <img ref={imgRef} src={imageSrc} className="absolute inset-0 w-full h-full object-contain pointer-events-none" alt="Screen Capture" />

      {/* Dimmed Overlay (shown when no selection) */}
      {!selection && <div className="absolute inset-0 bg-black opacity-40 pointer-events-none"></div>}

      {/* Selection Box with inverted shadow for dimming outside */}
      {selection && (
          <div
            className="absolute border-2 border-primary bg-transparent"
            style={{
                left: selection.x,
                top: selection.y,
                width: selection.w,
                height: selection.h,
                boxShadow: '0 0 0 9999px rgba(0, 0, 0, 0.5)'
            }}
          >
          </div>
      )}

      <div className="absolute top-4 left-4 bg-black/70 text-white px-3 py-1 rounded pointer-events-none">
          Drag to select area. ESC to cancel.
      </div>
    </div>
  );
}

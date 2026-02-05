import { useState, useEffect, useRef, MouseEvent } from 'react';
import { invoke } from "@tauri-apps/api/core";

interface OverlayProps {
  screenIndex: number;
  onClose: () => void;
}

export default function Overlay({ screenIndex, onClose }: OverlayProps) {
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [selection, setSelection] = useState<{x: number, y: number, w: number, h: number} | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [processing, setProcessing] = useState(false);
  const startPos = useRef<{x: number, y: number} | null>(null);
  const imgRef = useRef<HTMLImageElement>(null);

  useEffect(() => {
    async function fetchImage() {
      try {
        const result = await invoke<string>('get_overlay_image', { screenIndex });
        setImageSrc(result);
      } catch (e) {
        console.error("Failed to get overlay image:", e);
        alert("Failed to get overlay image: " + e);
        onClose();
      }
    }
    fetchImage();

    const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'Escape') handleClose();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [screenIndex, onClose]);

  const handleClose = async () => {
      try {
          await invoke('close_all_overlays');
          onClose();
      } catch (e) {
          console.error("Failed to close overlays", e);
          onClose();
      }
  };

  const handleMouseDown = (e: MouseEvent) => {
    if (processing) return;
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

  const handleMouseUp = async () => {
    setIsDragging(false);
    if (selection && selection.w > 10 && selection.h > 10 && imgRef.current) {
        // Calculate scaling
        const img = imgRef.current;
        const rect = img.getBoundingClientRect();
        const naturalWidth = img.naturalWidth;
        const naturalHeight = img.naturalHeight;

        if (naturalWidth === 0 || naturalHeight === 0) return;

        // Since we are fullscreen on the correct monitor, image should match window aspect ratio exactly (mostly).
        // But to be safe, we use the same robust logic or simplified.
        // Simplified:
        const scaleX = naturalWidth / rect.width;
        const scaleY = naturalHeight / rect.height;

        const realX = Math.round((selection.x - rect.left) * scaleX);
        const realY = Math.round((selection.y - rect.top) * scaleY);
        const realW = Math.round(selection.w * scaleX);
        const realH = Math.round(selection.h * scaleY);

        setProcessing(true);
        try {
            await invoke('process_selection', {
                screenIndex,
                x: realX,
                y: realY,
                w: realW,
                h: realH
            });
            // Calling handleClose will close all overlays and show main window
            handleClose();
        } catch (e) {
            console.error(e);
            alert("Error: " + e);
            setProcessing(false);
            setSelection(null);
        }
    } else {
        setSelection(null);
    }
  };

  if (!imageSrc) return <div className="fixed inset-0 bg-black text-white flex items-center justify-center z-50">Loading...</div>;

  return (
    <div
        className="fixed inset-0 cursor-crosshair z-50 overflow-hidden select-none"
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
    >
      <img ref={imgRef} src={imageSrc} className="absolute inset-0 w-full h-full object-contain pointer-events-none" alt="Screen Capture" />

      {!selection && <div className="absolute inset-0 bg-black opacity-10 pointer-events-none"></div>}

      {selection && (
          <div
            className="absolute border-2 border-primary bg-transparent"
            style={{
                left: selection.x,
                top: selection.y,
                width: selection.w,
                height: selection.h,
                boxShadow: '0 0 0 9999px rgba(0, 0, 0, 0.3)'
            }}
          >
          </div>
      )}

      {processing && (
          <div className="fixed inset-0 flex items-center justify-center z-[60] bg-black/20 text-white font-bold">
              Processing...
          </div>
      )}
    </div>
  );
}

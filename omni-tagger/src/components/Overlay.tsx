import { useState, useEffect, useRef, MouseEvent as ReactMouseEvent } from 'react';
import { invoke } from "@tauri-apps/api/core";

interface OverlayProps {
  screenIndex: number;
  onClose: () => void;
}

type Mode = 'none' | 'selecting' | 'selected' | 'moving' | 'resizing';
type ResizeHandle = 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw';

interface Selection {
  x: number;
  y: number;
  w: number;
  h: number;
}

export default function Overlay({ screenIndex, onClose }: OverlayProps) {
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [selection, setSelection] = useState<Selection | null>(null);
  const [mode, setMode] = useState<Mode>('none');
  const [processing, setProcessing] = useState(false);

  // Refs for drag operations
  const startPos = useRef<{x: number, y: number} | null>(null);
  const startSelection = useRef<Selection | null>(null);
  const activeHandle = useRef<ResizeHandle | null>(null);
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
        if (e.key === 'Escape') {
            if (mode === 'selected' || mode === 'none') {
                handleClose();
            } else {
                setSelection(null);
                setMode('none');
            }
        }
        if (e.key === 'Enter' && selection) {
            confirmSelection();
        }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [screenIndex, onClose, mode, selection]);

  const handleClose = async () => {
      try {
          await invoke('close_all_overlays');
          onClose();
      } catch (e) {
          console.error("Failed to close overlays", e);
          onClose();
      }
  };

  const confirmSelection = async () => {
    if (!selection || !imgRef.current) return;

    // Calculate scaling
    const img = imgRef.current;
    const rect = img.getBoundingClientRect();
    const naturalWidth = img.naturalWidth;
    const naturalHeight = img.naturalHeight;

    if (naturalWidth === 0 || naturalHeight === 0) return;

    // Calculate aspect ratios
    const imageAspect = naturalWidth / naturalHeight;
    const containerAspect = rect.width / rect.height;

    let displayWidth, displayHeight;
    let offsetLeft = 0;
    let offsetTop = 0;

    if (imageAspect > containerAspect) {
        // Image is wider relative to container: fits width, letterboxed height
        displayWidth = rect.width;
        displayHeight = rect.width / imageAspect;
        offsetLeft = 0;
        offsetTop = (rect.height - displayHeight) / 2;
    } else {
        // Image is taller relative to container: fits height, pillared width
        displayHeight = rect.height;
        displayWidth = rect.height * imageAspect;
        offsetTop = 0;
        offsetLeft = (rect.width - displayWidth) / 2;
    }

    const scaleX = naturalWidth / displayWidth;
    const scaleY = naturalHeight / displayHeight;

    // Adjust selection coordinates to be relative to the displayed image
    const relativeX = selection.x - rect.left - offsetLeft;
    const relativeY = selection.y - rect.top - offsetTop;

    const realX = Math.round(relativeX * scaleX);
    const realY = Math.round(relativeY * scaleY);
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
        handleClose();
    } catch (e) {
        console.error(e);
        alert("Error: " + e);
        setProcessing(false);
    }
  };

  const handleMouseDown = (e: ReactMouseEvent) => {
    if (processing) return;
    if (e.button !== 0) return; // Only left click

    const clientX = e.clientX;
    const clientY = e.clientY;

    // Check if clicking on a handle (handled by specific onMouseDown on handles)
    // If not on a handle, check if inside selection
    if (selection && mode === 'selected') {
        if (clientX >= selection.x && clientX <= selection.x + selection.w &&
            clientY >= selection.y && clientY <= selection.y + selection.h) {

            // Start moving
            setMode('moving');
            startPos.current = { x: clientX, y: clientY };
            startSelection.current = { ...selection };
            return;
        }
    }

    // Start new selection
    setMode('selecting');
    setSelection({ x: clientX, y: clientY, w: 0, h: 0 });
    startPos.current = { x: clientX, y: clientY };
  };

  const handleHandleMouseDown = (e: ReactMouseEvent, handle: ResizeHandle) => {
    e.stopPropagation(); // Prevent triggering background mousedown
    if (processing || !selection) return;

    setMode('resizing');
    activeHandle.current = handle;
    startPos.current = { x: e.clientX, y: e.clientY };
    startSelection.current = { ...selection };
  };

  const handleMouseMove = (e: ReactMouseEvent) => {
    if (mode === 'none' || !startPos.current) return;

    const currentX = e.clientX;
    const currentY = e.clientY;

    if (mode === 'selecting') {
        const x = Math.min(startPos.current.x, currentX);
        const y = Math.min(startPos.current.y, currentY);
        const w = Math.abs(currentX - startPos.current.x);
        const h = Math.abs(currentY - startPos.current.y);
        setSelection({ x, y, w, h });
    } else if (mode === 'moving' && startSelection.current) {
        const dx = currentX - startPos.current.x;
        const dy = currentY - startPos.current.y;
        setSelection({
            ...startSelection.current,
            x: startSelection.current.x + dx,
            y: startSelection.current.y + dy
        });
    } else if (mode === 'resizing' && startSelection.current && activeHandle.current) {
        const dx = currentX - startPos.current.x;
        const dy = currentY - startPos.current.y;
        const s = startSelection.current;
        let newX = s.x;
        let newY = s.y;
        let newW = s.w;
        let newH = s.h;

        const h = activeHandle.current;

        if (h.includes('e')) newW = Math.max(10, s.w + dx);
        if (h.includes('s')) newH = Math.max(10, s.h + dy);

        if (h.includes('w')) {
            const w = Math.max(10, s.w - dx);
            newX = s.x + (s.w - w);
            newW = w;
        }
        if (h.includes('n')) {
            const h = Math.max(10, s.h - dy);
            newY = s.y + (s.h - h);
            newH = h;
        }

        setSelection({ x: newX, y: newY, w: newW, h: newH });
    }
  };

  const handleMouseUp = () => {
    if (mode === 'selecting') {
        if (selection && selection.w > 10 && selection.h > 10) {
            setMode('selected');
        } else {
            setSelection(null);
            setMode('none');
        }
    } else if (mode === 'moving' || mode === 'resizing') {
        setMode('selected');
    }

    startPos.current = null;
    startSelection.current = null;
    activeHandle.current = null;
  };

  if (!imageSrc) return <div className="fixed inset-0 bg-black text-white flex items-center justify-center z-50">Loading...</div>;

  return (
    <div
        className="fixed inset-0 z-50 overflow-hidden select-none"
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        style={{ cursor: mode === 'selecting' ? 'crosshair' : 'default' }}
    >
      <img ref={imgRef} src={imageSrc} className="absolute inset-0 w-full h-full object-contain pointer-events-none" alt="Screen Capture" />

      {/* Dimmed background */}
      <div className="absolute inset-0 bg-black opacity-30 pointer-events-none"
           style={{
               clipPath: selection ? `polygon(0% 0%, 0% 100%, ${selection.x}px 100%, ${selection.x}px ${selection.y}px, ${selection.x + selection.w}px ${selection.y}px, ${selection.x + selection.w}px ${selection.y + selection.h}px, ${selection.x}px ${selection.y + selection.h}px, ${selection.x}px 100%, 100% 100%, 100% 0%)` : 'none'
           }}>
      </div>

      {selection && (
          <div
            className="absolute border-2 border-blue-500 bg-transparent"
            style={{
                left: selection.x,
                top: selection.y,
                width: selection.w,
                height: selection.h,
                cursor: mode === 'selected' ? 'move' : 'default'
            }}
          >
              {/* Resize Handles */}
              {mode === 'selected' && (
                  <>
                    <div onMouseDown={(e) => handleHandleMouseDown(e, 'nw')} className="absolute -top-1.5 -left-1.5 w-3 h-3 bg-white border border-blue-500 cursor-nw-resize" />
                    <div onMouseDown={(e) => handleHandleMouseDown(e, 'n')} className="absolute -top-1.5 left-1/2 -translate-x-1/2 w-3 h-3 bg-white border border-blue-500 cursor-n-resize" />
                    <div onMouseDown={(e) => handleHandleMouseDown(e, 'ne')} className="absolute -top-1.5 -right-1.5 w-3 h-3 bg-white border border-blue-500 cursor-ne-resize" />
                    <div onMouseDown={(e) => handleHandleMouseDown(e, 'w')} className="absolute top-1/2 -translate-y-1/2 -left-1.5 w-3 h-3 bg-white border border-blue-500 cursor-w-resize" />
                    <div onMouseDown={(e) => handleHandleMouseDown(e, 'e')} className="absolute top-1/2 -translate-y-1/2 -right-1.5 w-3 h-3 bg-white border border-blue-500 cursor-e-resize" />
                    <div onMouseDown={(e) => handleHandleMouseDown(e, 'sw')} className="absolute -bottom-1.5 -left-1.5 w-3 h-3 bg-white border border-blue-500 cursor-sw-resize" />
                    <div onMouseDown={(e) => handleHandleMouseDown(e, 's')} className="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-3 h-3 bg-white border border-blue-500 cursor-s-resize" />
                    <div onMouseDown={(e) => handleHandleMouseDown(e, 'se')} className="absolute -bottom-1.5 -right-1.5 w-3 h-3 bg-white border border-blue-500 cursor-se-resize" />
                  </>
              )}

              {/* Action Buttons */}
              {mode === 'selected' && (
                  <div className="absolute -bottom-12 right-0 flex space-x-2">
                      <button
                        className="bg-red-500 hover:bg-red-600 text-white p-2 rounded-full shadow-lg"
                        onClick={(e) => {
                            e.stopPropagation();
                            setSelection(null);
                            setMode('none');
                        }}
                        title="Cancel"
                      >
                          <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                            <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                          </svg>
                      </button>
                      <button
                        className="bg-green-500 hover:bg-green-600 text-white p-2 rounded-full shadow-lg"
                        onClick={(e) => {
                            e.stopPropagation();
                            confirmSelection();
                        }}
                        title="Extract Tags"
                      >
                          <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                            <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                          </svg>
                      </button>
                  </div>
              )}
          </div>
      )}

      {processing && (
          <div className="fixed inset-0 flex items-center justify-center z-[60] bg-black/50 text-white font-bold text-xl">
              Processing...
          </div>
      )}
    </div>
  );
}

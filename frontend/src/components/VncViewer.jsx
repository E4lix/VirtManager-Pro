import React, { useEffect, useRef, useState } from 'react';
import { Monitor, Maximize2, RefreshCw, Power, Terminal } from 'lucide-react';
import RFB from '@novnc/novnc';

export default function VncViewer({ vmId, vmName }) {
  const containerRef = useRef(null);
  const rfbRef = useRef(null);
  const [status, setStatus] = useState('Connexion en cours...');
  const [isConnected, setIsConnected] = useState(false);
  const [useIframeFallback, setUseIframeFallback] = useState(false);

  useEffect(() => {
    let rfb = null;
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/vnc/ws/${vmId}`;

    if (containerRef.current && !useIframeFallback) {
      try {
        rfb = new RFB(containerRef.current, wsUrl, {
          credentials: { password: '' },
          wsProtocols: ['binary'],
        });

        rfb.viewOnly = false;
        rfb.focusOnClick = true;
        rfb.scaleViewport = true;
        rfb.resizeSession = true;

        rfb.addEventListener('connect', () => {
          setStatus('Connecté à la machine virtuelle');
          setIsConnected(true);
        });

        rfb.addEventListener('disconnect', (e) => {
          setIsConnected(false);
          setStatus(e.detail.clean ? 'Session terminée' : 'Connexion fermée');
        });

        rfb.addEventListener('securityfailure', () => {
          setStatus('Échec de sécurité VNC');
        });

        rfbRef.current = rfb;
      } catch (err) {
        console.warn('noVNC direct canvas init failed, switching to iframe viewer:', err);
        setUseIframeFallback(true);
      }
    }

    return () => {
      if (rfb) {
        try {
          rfb.disconnect();
        } catch (_) {}
      }
    };
  }, [vmId, useIframeFallback]);

  const handleSendCtrlAltDel = () => {
    if (rfbRef.current) {
      rfbRef.current.sendCtrlAltDel();
    }
  };

  const handleFullscreen = () => {
    if (containerRef.current) {
      if (!document.fullscreenElement) {
        containerRef.current.requestFullscreen().catch((err) => alert(err.message));
      } else {
        document.exitFullscreen();
      }
    }
  };

  const handleReconnect = () => {
    if (rfbRef.current) {
      try {
        rfbRef.current.disconnect();
      } catch (_) {}
    }
    setStatus('Reconnexion...');
    setIsConnected(false);
    setUseIframeFallback(!useIframeFallback);
  };

  const iframeUrl = `/novnc/vnc_lite.html?path=api/vnc/ws/${vmId}&autoconnect=true&resize=scale`;

  return (
    <div className="vnc-screen-container">
      {/* VNC Toolbar */}
      <div className="vnc-toolbar">
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
          <Monitor size={18} color="#38bdf8" />
          <span style={{ fontWeight: 600, fontSize: '0.9rem' }}>
            Console VNC : {vmName}
          </span>
          <span
            className={`badge ${isConnected ? 'badge-running' : 'badge-paused'}`}
            style={{ fontSize: '0.7rem' }}
          >
            <span className="status-dot"></span>
            {status}
          </span>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <button
            className="btn btn-secondary btn-sm"
            onClick={handleSendCtrlAltDel}
            title="Envoyer la séquence Ctrl+Alt+Suppr"
          >
            <Terminal size={14} />
            Ctrl+Alt+Suppr
          </button>
          <button
            className="btn btn-secondary btn-sm"
            onClick={handleReconnect}
            title="Basculer le mode de rendu"
          >
            <RefreshCw size={14} />
            Recharger
          </button>
          <button
            className="btn btn-secondary btn-sm"
            onClick={handleFullscreen}
            title="Plein écran"
          >
            <Maximize2 size={14} />
            Plein écran
          </button>
        </div>
      </div>

      {/* VNC Display Canvas Area */}
      <div className="vnc-canvas-area">
        {useIframeFallback ? (
          <iframe
            src={iframeUrl}
            title={`VNC Console ${vmName}`}
            style={{ width: '100%', height: '100%', border: 'none' }}
          />
        ) : (
          <div
            ref={containerRef}
            style={{
              width: '100%',
              height: '100%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          />
        )}
      </div>
    </div>
  );
}

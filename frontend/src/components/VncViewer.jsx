import React, { useEffect, useRef, useState } from 'react';
import { Monitor, Maximize2, RefreshCw, Terminal, ExternalLink, AlertTriangle } from 'lucide-react';
import RFB from '@novnc/novnc';

export default function VncViewer({ vmId, vmName }) {
  const containerRef = useRef(null);
  const rfbRef = useRef(null);
  const [status, setStatus] = useState('Connexion en cours...');
  const [isConnected, setIsConnected] = useState(false);
  const [useIframeFallback, setUseIframeFallback] = useState(false);
  const [connectError, setConnectError] = useState(null);

  const iframeUrl = `/novnc/vnc_lite.html?path=api/vnc/ws/${encodeURIComponent(vmId)}&autoconnect=true&resize=scale`;

  useEffect(() => {
    let isMounted = true;
    let rfb = null;
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/vnc/ws/${encodeURIComponent(vmId)}`;

    setConnectError(null);
    setStatus('Connexion en cours...');

    if (containerRef.current && !useIframeFallback) {
      containerRef.current.innerHTML = '';
      try {
        rfb = new RFB(containerRef.current, wsUrl, {
          credentials: { password: '' },
          wsProtocols: ['binary'],
        });

        rfb.viewOnly = false;
        rfb.focusOnClick = true;
        rfb.scaleViewport = true;
        // resizeSession is disabled to avoid unsupported SetDesktopSize requests on standard QEMU VNC
        rfb.resizeSession = false;

        rfb.addEventListener('connect', () => {
          if (!isMounted) return;
          setStatus('Connecté à la machine virtuelle');
          setIsConnected(true);
          setConnectError(null);
        });

        rfb.addEventListener('disconnect', (e) => {
          if (!isMounted) return;
          setIsConnected(false);
          const clean = e.detail?.clean;
          setStatus(clean ? 'Session terminée' : 'Connexion fermée');
          if (!clean) {
            setConnectError('La connexion avec le serveur VNC a été interrompue.');
          }
        });

        rfb.addEventListener('securityfailure', (e) => {
          if (!isMounted) return;
          setIsConnected(false);
          setStatus('Échec de sécurité VNC');
          setConnectError(`Échec d'authentification ou sécurité VNC (${e.detail?.status || 'inconnu'})`);
        });

        rfbRef.current = rfb;
      } catch (err) {
        console.warn('noVNC direct canvas init failed, falling back to iframe:', err);
        if (isMounted) {
          setUseIframeFallback(true);
        }
      }
    }

    return () => {
      isMounted = false;
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
    setConnectError(null);
    setStatus('Reconnexion...');
    setIsConnected(false);
    // Force re-render of current viewer mode
    if (useIframeFallback) {
      const iframe = document.getElementById('novnc-iframe');
      if (iframe) iframe.src = iframeUrl;
    } else {
      setUseIframeFallback(false);
    }
  };

  const handleToggleMode = () => {
    if (rfbRef.current) {
      try {
        rfbRef.current.disconnect();
      } catch (_) {}
    }
    setConnectError(null);
    setUseIframeFallback(!useIframeFallback);
  };

  const handleOpenPopout = () => {
    window.open(iframeUrl, `vnc-${vmId}`, 'width=1024,height=768,menubar=no,toolbar=no,location=no');
  };

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
          {useIframeFallback && (
            <span style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
              (Mode noVNC Lite)
            </span>
          )}
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
            title="Relancer la connexion"
          >
            <RefreshCw size={14} />
            Recharger
          </button>
          <button
            className="btn btn-secondary btn-sm"
            onClick={handleToggleMode}
            title={useIframeFallback ? "Basculer en mode Canvas direct" : "Basculer en mode Iframe noVNC"}
          >
            {useIframeFallback ? 'Mode Direct' : 'Mode noVNC Lite'}
          </button>
          <button
            className="btn btn-secondary btn-sm"
            onClick={handleOpenPopout}
            title="Ouvrir dans une fenêtre détachée"
          >
            <ExternalLink size={14} />
            Pop-out
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
      <div className="vnc-canvas-area" style={{ position: 'relative' }}>
        {useIframeFallback ? (
          <iframe
            id="novnc-iframe"
            src={iframeUrl}
            title={`VNC Console ${vmName}`}
            style={{ width: '100%', height: '100%', border: 'none' }}
          />
        ) : (
          <>
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
            {connectError && (
              <div
                style={{
                  position: 'absolute',
                  inset: 0,
                  backgroundColor: 'rgba(15, 23, 42, 0.85)',
                  backdropFilter: 'blur(4px)',
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '1rem',
                  padding: '2rem',
                  textAlign: 'center',
                  zIndex: 20,
                }}
              >
                <AlertTriangle size={36} color="#f59e0b" />
                <div style={{ fontSize: '1.1rem', fontWeight: 600, color: '#f8fafc' }}>
                  {status}
                </div>
                <div style={{ fontSize: '0.85rem', color: '#94a3b8', maxWidth: '450px' }}>
                  {connectError}
                </div>
                <div style={{ display: 'flex', gap: '0.75rem', marginTop: '0.5rem' }}>
                  <button className="btn btn-primary btn-sm" onClick={handleReconnect}>
                    <RefreshCw size={14} />
                    Réessayer
                  </button>
                  <button className="btn btn-secondary btn-sm" onClick={handleToggleMode}>
                    Basculer en mode noVNC Lite
                  </button>
                  <button className="btn btn-secondary btn-sm" onClick={handleOpenPopout}>
                    <ExternalLink size={14} />
                    Ouvrir en fenêtre externe
                  </button>
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

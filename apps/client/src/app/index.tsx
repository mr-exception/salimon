import { useCallback, useEffect, useRef, useState } from 'react';
import axios from 'axios';
import { Communications, Footer, Navigator, StartMenu } from '@components';
import {
  getApiBaseUrl,
  getStoredSpaceshipSecurityCode,
  SECURITY_CODE_HEADER,
  useBootstrap,
  type BootstrapRequest,
} from '@store';
import style from './style.module.css';

type UnreadMessage = {
  id: string;
  contactId: string;
  sender: 'player' | 'contact';
  text: string;
  status: 'sent' | 'queued' | 'failed';
  isRead: boolean;
  createdAt: string;
};

const UNREAD_POLL_MS = 30_000;

export default function App() {
  const [bootstrapRequest, setBootstrapRequest] =
    useState<BootstrapRequest | null>(null);
  const bootstrapState = useBootstrap(bootstrapRequest);
  const [isEngineRunning, setIsEngineRunning] = useState(false);
  const [isCommunicationsOpen, setIsCommunicationsOpen] = useState(false);
  const [unreadMessages, setUnreadMessages] = useState<UnreadMessage[]>([]);
  const [isSelectingTargetDirection, setIsSelectingTargetDirection] =
    useState(false);
  const sceneRef = useRef<{
    startEngines: (targetSpeed: number, maximumThrustPercent: number) => void;
    stopEngines: () => void;
    setManualThrust: (
      direction: { x: number; y: number } | undefined,
      power: number,
    ) => void;
    setTargetDirectionSelectionActive: (active: boolean) => void;
  } | null>(null);
  const handleSceneChange = useCallback(
    (
      scene: {
        startEngines: (
          targetSpeed: number,
          maximumThrustPercent: number,
        ) => void;
        stopEngines: () => void;
        setManualThrust: (
          direction: { x: number; y: number } | undefined,
          power: number,
        ) => void;
        setTargetDirectionSelectionActive: (active: boolean) => void;
      } | null,
    ) => {
      sceneRef.current = scene;
    },
    [],
  );
  const startEngines = useCallback(
    (targetSpeed: number, maximumThrustPercent: number) => {
      sceneRef.current?.startEngines(targetSpeed, maximumThrustPercent);
    },
    [],
  );
  const stopEngines = useCallback(() => {
    sceneRef.current?.stopEngines();
  }, []);
  const setManualThrust = useCallback(
    (direction: { x: number; y: number } | undefined, power: number) => {
      sceneRef.current?.setManualThrust(direction, power);
    },
    [],
  );
  const toggleTargetDirectionSelection = useCallback(() => {
    setIsSelectingTargetDirection((isSelecting) => {
      const active = !isSelecting;
      sceneRef.current?.setTargetDirectionSelectionActive(active);
      return active;
    });
  }, []);
  const handleTargetDirectionSelected = useCallback(() => {
    setIsSelectingTargetDirection(false);
  }, []);
  const handleMessagesRead = useCallback((messageIds: string[]) => {
    if (messageIds.length === 0) return;
    const readIds = new Set(messageIds);
    setUnreadMessages((current) =>
      current.filter((message) => !readIds.has(message.id)),
    );
  }, []);

  useEffect(() => {
    if (bootstrapState !== 'ready') return;
    const securityCode = getStoredSpaceshipSecurityCode();
    if (!securityCode) return;

    let disposed = false;
    let timer: number | undefined;
    const pollUnreadMessages = async () => {
      if (disposed) return;
      if (!document.hidden && navigator.onLine) {
        try {
          const { data } = await axios.get<{ messages: UnreadMessage[] }>(
            `${getApiBaseUrl()}/contacts/messages/unread`,
            { headers: { [SECURITY_CODE_HEADER]: securityCode } },
          );
          if (!disposed) setUnreadMessages(data.messages);
        } catch (error) {
          console.error('Failed to load unread messages', error);
        }
      }
      timer = window.setTimeout(pollUnreadMessages, UNREAD_POLL_MS);
    };
    const pollNow = () => {
      window.clearTimeout(timer);
      void pollUnreadMessages();
    };

    void pollUnreadMessages();
    document.addEventListener('visibilitychange', pollNow);
    window.addEventListener('online', pollNow);
    return () => {
      disposed = true;
      window.clearTimeout(timer);
      document.removeEventListener('visibilitychange', pollNow);
      window.removeEventListener('online', pollNow);
    };
  }, [bootstrapState]);

  if (bootstrapState !== 'ready') {
    return (
      <StartMenu
        bootstrapState={bootstrapState}
        onStart={setBootstrapRequest}
      />
    );
  }

  return (
    <div className={style.app}>
      <Navigator
        onSceneChange={handleSceneChange}
        onSpaceshipEngineChange={setIsEngineRunning}
        isSelectingTargetDirection={isSelectingTargetDirection}
        onTargetDirectionSelected={handleTargetDirectionSelected}
      />
      <Footer
        isEngineRunning={isEngineRunning}
        onStartEngines={startEngines}
        onStopEngines={stopEngines}
        onManualThrustChange={setManualThrust}
        onOpenCommunications={() => setIsCommunicationsOpen(true)}
        unreadMessageCount={unreadMessages.length}
        isSelectingTargetDirection={isSelectingTargetDirection}
        onToggleTargetDirectionSelection={toggleTargetDirectionSelection}
      />
      {isCommunicationsOpen && (
        <Communications
          unreadMessages={unreadMessages}
          onMessagesRead={handleMessagesRead}
          onClose={() => setIsCommunicationsOpen(false)}
        />
      )}
    </div>
  );
}

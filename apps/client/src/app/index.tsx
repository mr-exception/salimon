import { useCallback, useEffect, useRef, useState } from 'react';
import axios from 'axios';
import { Communications, Footer, Navigator, StartMenu } from '@components';
import {
  getApiBaseUrl,
  getStoredSpaceshipSecurityCode,
  SECURITY_CODE_HEADER,
  subscribeToContactMessages,
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

export default function App() {
  const [bootstrapRequest, setBootstrapRequest] =
    useState<BootstrapRequest | null>(null);
  const bootstrapState = useBootstrap(bootstrapRequest);
  const [isEngineRunning, setIsEngineRunning] = useState(false);
  const [isCommunicationsOpen, setIsCommunicationsOpen] = useState(false);
  const [isMeasuring, setIsMeasuring] = useState(false);
  const [isRulerActive, setIsRulerActive] = useState(false);
  const [unreadMessages, setUnreadMessages] = useState<UnreadMessage[]>([]);
  const [isSelectingTargetDirection, setIsSelectingTargetDirection] =
    useState(false);
  const sceneRef = useRef<{
    startThrusters: (
      thrusters: { powerPercent: number; active: boolean }[],
    ) => void;
    stopEngines: () => void;
    setTargetDirectionSelectionActive: (active: boolean) => void;
    setRulerActive: (active: boolean) => void;
    setPrediction: (active: boolean, seconds: number) => void;
  } | null>(null);
  const handleSceneChange = useCallback(
    (
      scene: {
        startThrusters: (
          thrusters: { powerPercent: number; active: boolean }[],
        ) => void;
        stopEngines: () => void;
        setTargetDirectionSelectionActive: (active: boolean) => void;
        setRulerActive: (active: boolean) => void;
        setPrediction: (active: boolean, seconds: number) => void;
      } | null,
    ) => {
      sceneRef.current = scene;
    },
    [],
  );
  const startThrusters = useCallback(
    (thrusters: { powerPercent: number; active: boolean }[]) => {
      sceneRef.current?.startThrusters(thrusters);
    },
    [],
  );
  const stopEngines = useCallback(() => {
    sceneRef.current?.stopEngines();
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
  const setPrediction = useCallback((active: boolean, seconds: number) => {
    sceneRef.current?.setPrediction(active, seconds);
  }, []);

  useEffect(() => {
    if (bootstrapState !== 'ready') return;
    const securityCode = getStoredSpaceshipSecurityCode();
    if (!securityCode) return;

    let disposed = false;
    const loadUnreadMessages = async () => {
      if (disposed) return;
      try {
        const { data } = await axios.get<{ messages: UnreadMessage[] }>(
          `${getApiBaseUrl()}/contacts/messages/unread`,
          { headers: { [SECURITY_CODE_HEADER]: securityCode } },
        );
        if (!disposed) setUnreadMessages(data.messages);
      } catch (error) {
        console.error('Failed to load unread messages', error);
      }
    };
    const refreshUnreadMessages = () => {
      if (!document.hidden && navigator.onLine) void loadUnreadMessages();
    };
    const unreadRefreshTimer = window.setInterval(
      refreshUnreadMessages,
      15_000,
    );
    const unsubscribe = subscribeToContactMessages((message) => {
      if (message.sender !== 'contact') return;
      setUnreadMessages((current) => {
        if (
          current.some((currentMessage) => currentMessage.id === message.id)
        ) {
          return current;
        }
        return [...current, message].sort(
          (left, right) =>
            Date.parse(left.createdAt) - Date.parse(right.createdAt),
        );
      });
    });

    void loadUnreadMessages();
    document.addEventListener('visibilitychange', refreshUnreadMessages);
    window.addEventListener('online', refreshUnreadMessages);
    return () => {
      disposed = true;
      unsubscribe();
      window.clearInterval(unreadRefreshTimer);
      document.removeEventListener('visibilitychange', refreshUnreadMessages);
      window.removeEventListener('online', refreshUnreadMessages);
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
        isMeasuring={isMeasuring}
        isRulerActive={isRulerActive}
        onSceneChange={handleSceneChange}
        onSpaceshipEngineChange={setIsEngineRunning}
        isSelectingTargetDirection={isSelectingTargetDirection}
        onTargetDirectionSelected={handleTargetDirectionSelected}
      />
      <Footer
        isEngineRunning={isEngineRunning}
        isMeasuring={isMeasuring}
        isRulerActive={isRulerActive}
        onStartThrusters={startThrusters}
        onStopEngines={stopEngines}
        onToggleMeasuring={() => setIsMeasuring((active) => !active)}
        onToggleRuler={() => setIsRulerActive((active) => !active)}
        onOpenCommunications={() => setIsCommunicationsOpen(true)}
        unreadMessageCount={unreadMessages.length}
        onPredictionChange={setPrediction}
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

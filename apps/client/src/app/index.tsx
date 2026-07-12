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
  const [unreadMessages, setUnreadMessages] = useState<UnreadMessage[]>([]);
  const [isSelectingTargetDirection, setIsSelectingTargetDirection] =
    useState(false);
  const sceneRef = useRef<{
    startEngines: (targetSpeed: number, maximumThrustPercent: number) => void;
    startManualForce: (
      thrusters: { powerPercent: number; durationSeconds: number }[],
    ) => void;
    stopEngines: () => void;
    setTargetDirectionSelectionActive: (active: boolean) => void;
    setPrediction: (active: boolean, seconds: number) => void;
  } | null>(null);
  const handleSceneChange = useCallback(
    (
      scene: {
        startEngines: (
          targetSpeed: number,
          maximumThrustPercent: number,
        ) => void;
        startManualForce: (
          thrusters: { powerPercent: number; durationSeconds: number }[],
        ) => void;
        stopEngines: () => void;
        setTargetDirectionSelectionActive: (active: boolean) => void;
        setPrediction: (active: boolean, seconds: number) => void;
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
  const startManualForce = useCallback(
    (thrusters: { powerPercent: number; durationSeconds: number }[]) => {
      sceneRef.current?.startManualForce(thrusters);
    },
    [],
  );
  const stopEngines = useCallback(() => {
    sceneRef.current?.stopEngines();
  }, []);
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
    const unsubscribe = subscribeToContactMessages((message) => {
      if (message.sender !== 'contact') return;
      setUnreadMessages((current) => {
        if (current.some((currentMessage) => currentMessage.id === message.id)) {
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
        onSceneChange={handleSceneChange}
        onSpaceshipEngineChange={setIsEngineRunning}
        isSelectingTargetDirection={isSelectingTargetDirection}
        onTargetDirectionSelected={handleTargetDirectionSelected}
      />
      <Footer
        isEngineRunning={isEngineRunning}
        isMeasuring={isMeasuring}
        onStartEngines={startEngines}
        onStartManualForce={startManualForce}
        onStopEngines={stopEngines}
        onToggleMeasuring={() => setIsMeasuring((active) => !active)}
        onOpenCommunications={() => setIsCommunicationsOpen(true)}
        unreadMessageCount={unreadMessages.length}
        isSelectingTargetDirection={isSelectingTargetDirection}
        onToggleTargetDirectionSelection={toggleTargetDirectionSelection}
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

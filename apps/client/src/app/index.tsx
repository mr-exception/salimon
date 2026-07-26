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
import type { ContactInfo } from '@repo/types';
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
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [isMeasuring, setIsMeasuring] = useState(false);
  const [
    isMeasurementRelativeToSpaceship,
    setIsMeasurementRelativeToSpaceship,
  ] = useState(false);
  const [isRulerActive, setIsRulerActive] = useState(false);
  const [initialCommunicationContactId, setInitialCommunicationContactId] =
    useState<string>();
  const [unreadMessages, setUnreadMessages] = useState<UnreadMessage[]>([]);
  const [contactNameById, setContactNameById] = useState<
    Record<string, string>
  >({});
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
  const openCommunicationThread = useCallback((contactId: string) => {
    setInitialCommunicationContactId(contactId);
    setIsCommunicationsOpen(true);
  }, []);
  const openCommunications = useCallback(() => {
    setInitialCommunicationContactId(undefined);
    setIsCommunicationsOpen(true);
  }, []);
  const setPrediction = useCallback((active: boolean, seconds: number) => {
    sceneRef.current?.setPrediction(active, seconds);
  }, []);

  useEffect(() => {
    if (bootstrapState !== 'ready') return;
    const securityCode = getStoredSpaceshipSecurityCode();
    if (!securityCode) return;

    let disposed = false;
    const loadContacts = async () => {
      if (disposed) return;
      try {
        const { data } = await axios.get<{ contacts: ContactInfo[] }>(
          `${getApiBaseUrl()}/contacts/info`,
          { headers: { [SECURITY_CODE_HEADER]: securityCode } },
        );
        if (!disposed) {
          setContactNameById(
            Object.fromEntries(
              data.contacts.map((contact) => [contact.id, contact.name]),
            ),
          );
        }
      } catch (error) {
        console.error('Failed to load contact info', error);
      }
    };
    const refreshContacts = () => {
      if (!document.hidden && navigator.onLine) void loadContacts();
    };
    const unsubscribe = subscribeToContactMessages((message) => {
      void loadContacts();
      if (message.sender !== 'contact' || message.isRead) return;
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

    void loadContacts();
    document.addEventListener('visibilitychange', refreshContacts);
    window.addEventListener('online', refreshContacts);
    return () => {
      disposed = true;
      unsubscribe();
      document.removeEventListener('visibilitychange', refreshContacts);
      window.removeEventListener('online', refreshContacts);
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
        isMeasurementRelativeToSpaceship={isMeasurementRelativeToSpaceship}
        isRulerActive={isRulerActive}
        isSearchOpen={isSearchOpen}
        onSceneChange={handleSceneChange}
        onSpaceshipEngineChange={setIsEngineRunning}
        onCloseSearch={() => setIsSearchOpen(false)}
        isSelectingTargetDirection={isSelectingTargetDirection}
        onTargetDirectionSelected={handleTargetDirectionSelected}
      />
      <Footer
        isEngineRunning={isEngineRunning}
        isMeasuring={isMeasuring}
        isMeasurementRelativeToSpaceship={isMeasurementRelativeToSpaceship}
        isRulerActive={isRulerActive}
        onStartThrusters={startThrusters}
        onStopEngines={stopEngines}
        onToggleMeasuring={() => setIsMeasuring((active) => !active)}
        onMeasurementRelativeToSpaceshipChange={
          setIsMeasurementRelativeToSpaceship
        }
        onToggleRuler={() => setIsRulerActive((active) => !active)}
        onOpenCommunications={openCommunications}
        onOpenCommunicationThread={openCommunicationThread}
        onOpenSearch={() => setIsSearchOpen(true)}
        unreadMessageCount={unreadMessages.length}
        unreadMessages={unreadMessages.map((message) => ({
          id: message.id,
          contactId: message.contactId,
          senderName: contactNameById[message.contactId] ?? 'Unknown contact',
          text: message.text,
        }))}
        onPredictionChange={setPrediction}
      />
      {isCommunicationsOpen && (
        <Communications
          key={initialCommunicationContactId ?? 'communications'}
          initialContactId={initialCommunicationContactId}
          unreadMessages={unreadMessages}
          onMessagesRead={handleMessagesRead}
          onClose={() => setIsCommunicationsOpen(false)}
        />
      )}
    </div>
  );
}

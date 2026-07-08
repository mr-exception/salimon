import type { Filter } from 'mongodb';
import { DatabaseModel } from './database.model';

export type ContactMessageSender = 'player' | 'contact';
export type ContactMessageStatus = 'sent' | 'queued' | 'failed';

export type ContactMessageDocument = {
  _id: string;
  spaceshipSecurityCode: string;
  contactId: string;
  sender: ContactMessageSender;
  text: string;
  status: ContactMessageStatus;
  isRead: boolean;
  clientMessageId?: string;
  createdAt: Date;
};

let indexesPromise: Promise<unknown> | undefined;

export class ContactMessageModel {
  static async getCollection() {
    const collection = (
      await DatabaseModel.getDatabase()
    ).collection<ContactMessageDocument>('contactMessages');
    indexesPromise ??= collection.createIndexes([
      {
        key: { spaceshipSecurityCode: 1, contactId: 1, createdAt: 1, _id: 1 },
      },
      {
        key: {
          spaceshipSecurityCode: 1,
          sender: 1,
          isRead: 1,
          createdAt: 1,
        },
      },
      {
        key: { spaceshipSecurityCode: 1, contactId: 1, clientMessageId: 1 },
        name: 'unique_client_message',
        unique: true,
        partialFilterExpression: { clientMessageId: { $type: 'string' } },
      },
    ]);
    await indexesPromise;
    return collection;
  }

  static async upsertInitialMessage(message: ContactMessageDocument) {
    return (await ContactMessageModel.getCollection()).updateOne(
      {
        spaceshipSecurityCode: message.spaceshipSecurityCode,
        contactId: message.contactId,
        sender: message.sender,
        clientMessageId: message.clientMessageId,
      },
      { $setOnInsert: message },
      { upsert: true },
    );
  }

  static async findLatest(
    spaceshipSecurityCode: string,
    contactId: string,
  ) {
    return (await ContactMessageModel.getCollection()).findOne(
      { spaceshipSecurityCode, contactId },
      { sort: { createdAt: -1, _id: -1 } },
    );
  }

  static async countUnreadContactMessages(
    spaceshipSecurityCode: string,
    contactId: string,
  ) {
    return (await ContactMessageModel.getCollection()).countDocuments({
      spaceshipSecurityCode,
      contactId,
      sender: 'contact',
      isRead: { $ne: true },
    });
  }

  static async findMessages(
    filter: Filter<ContactMessageDocument>,
    limit: number,
  ) {
    return (await ContactMessageModel.getCollection())
      .find(filter)
      .sort({ createdAt: 1, _id: 1 })
      .limit(limit)
      .toArray();
  }

  static async markMessagesRead(
    spaceshipSecurityCode: string,
    contactId: string,
    messageIds: string[],
  ) {
    return (await ContactMessageModel.getCollection()).updateMany(
      {
        spaceshipSecurityCode,
        contactId,
        _id: { $in: messageIds },
      },
      { $set: { isRead: true } },
    );
  }

  static async findUnreadContactMessages(spaceshipSecurityCode: string) {
    return (await ContactMessageModel.getCollection())
      .find({
        spaceshipSecurityCode,
        sender: 'contact',
        isRead: { $ne: true },
      })
      .sort({ createdAt: 1, _id: 1 })
      .toArray();
  }

  static async findByClientMessage(
    spaceshipSecurityCode: string,
    contactId: string,
    clientMessageId: string,
  ) {
    return (await ContactMessageModel.getCollection()).findOne({
      spaceshipSecurityCode,
      contactId,
      clientMessageId,
    });
  }

  static async countRecentPlayerMessages(
    spaceshipSecurityCode: string,
    after: Date,
  ) {
    return (await ContactMessageModel.getCollection()).countDocuments({
      spaceshipSecurityCode,
      sender: 'player',
      createdAt: { $gt: after },
    });
  }

  static async insert(message: ContactMessageDocument) {
    return (await ContactMessageModel.getCollection()).insertOne(message);
  }

  static async updateStatus(messageId: string, status: ContactMessageStatus) {
    return (await ContactMessageModel.getCollection()).updateOne(
      { _id: messageId },
      { $set: { status } },
    );
  }

  static async findPlayerMessage(
    spaceshipSecurityCode: string,
    contactId: string,
    playerMessageId: string,
  ) {
    return (await ContactMessageModel.getCollection()).findOne({
      _id: playerMessageId,
      spaceshipSecurityCode,
      contactId,
      sender: 'player',
    });
  }
}

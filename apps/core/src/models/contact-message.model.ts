import {
  getModelForClass,
  index,
  modelOptions,
  prop,
} from '@typegoose/typegoose';
import { DatabaseModel } from './database.model';

export type ContactMessageSender = 'player' | 'contact';
export type ContactMessageStatus = 'sent' | 'queued' | 'failed';

@index({ spaceshipSecurityCode: 1, contactId: 1, createdAt: 1, _id: 1 })
@index({ spaceshipSecurityCode: 1, sender: 1, isRead: 1, createdAt: 1 })
@index(
  { spaceshipSecurityCode: 1, contactId: 1, clientMessageId: 1 },
  {
    name: 'unique_client_message',
    unique: true,
    partialFilterExpression: { clientMessageId: { $type: 'string' } },
  },
)
@modelOptions({
  schemaOptions: { collection: 'contactMessages', versionKey: false },
})
class ContactMessageSchema {
  @prop({ required: true, type: () => String })
  public _id!: string;

  @prop({ required: true, type: () => String })
  public spaceshipSecurityCode!: string;

  @prop({ required: true, type: () => String })
  public contactId!: string;

  @prop({ required: true, enum: ['player', 'contact'], type: () => String })
  public sender!: ContactMessageSender;

  @prop({ required: true, type: () => String })
  public text!: string;

  @prop({
    required: true,
    enum: ['sent', 'queued', 'failed'],
    type: () => String,
  })
  public status!: ContactMessageStatus;

  @prop({ required: true, type: () => Boolean })
  public isRead!: boolean;

  @prop({ type: () => String })
  public clientMessageId?: string;

  @prop({ required: true, type: () => Date })
  public createdAt!: Date;
}

export type ContactMessageDocument = ContactMessageSchema;

const ContactMessageTypegooseModel = getModelForClass(ContactMessageSchema);

export class ContactMessageModel {
  static async getModel() {
    await DatabaseModel.connect();
    return ContactMessageTypegooseModel;
  }

  static async upsertInitialMessage(message: ContactMessageDocument) {
    return (await ContactMessageModel.getModel()).updateOne(
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

  static async findLatest(spaceshipSecurityCode: string, contactId: string) {
    return (await ContactMessageModel.getModel())
      .findOne({ spaceshipSecurityCode, contactId })
      .sort({ createdAt: -1, _id: -1 })
      .lean<ContactMessageDocument>()
      .exec();
  }

  static async countUnreadContactMessages(
    spaceshipSecurityCode: string,
    contactId: string,
  ) {
    return (await ContactMessageModel.getModel()).countDocuments({
      spaceshipSecurityCode,
      contactId,
      sender: 'contact',
      isRead: { $ne: true },
    });
  }

  static async findMessages(filter: Record<string, unknown>, limit: number) {
    return (await ContactMessageModel.getModel())
      .find(filter)
      .sort({ createdAt: 1, _id: 1 })
      .limit(limit)
      .lean<ContactMessageDocument[]>()
      .exec();
  }

  static async markMessagesRead(
    spaceshipSecurityCode: string,
    contactId: string,
    messageIds: string[],
  ) {
    return (await ContactMessageModel.getModel()).updateMany(
      {
        spaceshipSecurityCode,
        contactId,
        _id: { $in: messageIds },
      },
      { $set: { isRead: true } },
    );
  }

  static async findUnreadContactMessages(spaceshipSecurityCode: string) {
    return (await ContactMessageModel.getModel())
      .find({
        spaceshipSecurityCode,
        sender: 'contact',
        isRead: { $ne: true },
      })
      .sort({ createdAt: 1, _id: 1 })
      .lean<ContactMessageDocument[]>()
      .exec();
  }

  static async findByClientMessage(
    spaceshipSecurityCode: string,
    contactId: string,
    clientMessageId: string,
  ) {
    return (await ContactMessageModel.getModel())
      .findOne({
        spaceshipSecurityCode,
        contactId,
        clientMessageId,
      })
      .lean<ContactMessageDocument>()
      .exec();
  }

  static async countRecentPlayerMessages(
    spaceshipSecurityCode: string,
    after: Date,
  ) {
    return (await ContactMessageModel.getModel()).countDocuments({
      spaceshipSecurityCode,
      sender: 'player',
      createdAt: { $gt: after },
    });
  }

  static async insert(message: ContactMessageDocument) {
    return (await ContactMessageModel.getModel()).create(message);
  }

  static async updateStatus(messageId: string, status: ContactMessageStatus) {
    return (await ContactMessageModel.getModel()).updateOne(
      { _id: messageId },
      { $set: { status } },
    );
  }

  static async findPlayerMessage(
    spaceshipSecurityCode: string,
    contactId: string,
    playerMessageId: string,
  ) {
    return (await ContactMessageModel.getModel())
      .findOne({
        _id: playerMessageId,
        spaceshipSecurityCode,
        contactId,
        sender: 'player',
      })
      .lean<ContactMessageDocument>()
      .exec();
  }

  static async findReplyContext(
    spaceshipSecurityCode: string,
    contactId: string,
    limit: number,
  ) {
    return (await ContactMessageModel.getModel())
      .find({
        spaceshipSecurityCode,
        contactId,
        status: { $ne: 'failed' },
      })
      .sort({ createdAt: -1, _id: -1 })
      .limit(limit)
      .lean<ContactMessageDocument[]>()
      .exec();
  }
}

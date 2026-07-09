import {
  getModelForClass,
  index,
  modelOptions,
  prop,
} from '@typegoose/typegoose';
import { DatabaseModel } from './database.model';

@index({ spaceshipSecurityCode: 1, contactId: 1 }, { unique: true })
@modelOptions({ schemaOptions: { collection: 'contacts', versionKey: false } })
class ContactSchema {
  @prop({ required: true, type: () => String })
  public spaceshipSecurityCode!: string;

  @prop({ required: true, type: () => String })
  public contactId!: string;

  @prop({ required: true, type: () => Date })
  public unlockedAt!: Date;

  @prop({ type: () => Date })
  public lastReadAt?: Date;
}

export type ContactDocument = ContactSchema;

const ContactTypegooseModel = getModelForClass(ContactSchema);

export class ContactModel {
  static async getModel() {
    await DatabaseModel.connect();
    return ContactTypegooseModel;
  }

  static async findBySpaceshipSecurityCode(spaceshipSecurityCode: string) {
    return (await ContactModel.getModel())
      .find({ spaceshipSecurityCode })
      .lean<ContactDocument[]>()
      .exec();
  }

  static async findBySpaceshipAndContact(
    spaceshipSecurityCode: string,
    contactId: string,
  ) {
    return (await ContactModel.getModel())
      .findOne({ spaceshipSecurityCode, contactId })
      .lean<ContactDocument>()
      .exec();
  }

  static async upsertSpaceshipContact(contact: ContactDocument) {
    return (await ContactModel.getModel()).updateOne(
      {
        spaceshipSecurityCode: contact.spaceshipSecurityCode,
        contactId: contact.contactId,
      },
      { $setOnInsert: contact },
      { upsert: true },
    );
  }
}

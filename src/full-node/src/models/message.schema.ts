import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';
import { IMessage } from 'datamodels/message';
import { Entity } from './entity.schema';
import { Field, Int, ObjectType } from '@nestjs/graphql';

export type MessageDocument = Message & Document;

@ObjectType()
@Schema()
export class Message extends Entity implements IMessage {
  @Prop(String)
  @Field()
  messageId: string;
  @Prop(String)
  @Field()
  dstAddress: string;
  @Prop(String)
  @Field()
  dataPath: string;
  @Prop(Number)
  @Field()
  packetCount: number;
  @Prop(String)
  @Field()
  srcAddress: string;
  @Prop([Number])
  @Field(() => [Int])
  packetsOrder: number[];
}

export const MessageSchema = SchemaFactory.createForClass(Message);

export function messageResponse(message: Message): IMessage {
  return {
    _id: message._id,
    messageId: message.messageId,
    dstAddress: message.dstAddress,
    srcAddress: message.srcAddress,
    packetsOrder: message.packetsOrder,
    dataPath: message.dataPath,
    packetCount: message.packetCount,
    createdAt: message.createdAt,
    updatedAt: message.updatedAt,
  };
}

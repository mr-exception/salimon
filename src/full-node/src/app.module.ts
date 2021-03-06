import { ApolloDriver, ApolloDriverConfig } from '@nestjs/apollo';
import { Module } from '@nestjs/common';
import { ConfigModule } from '@nestjs/config';
import { GraphQLModule } from '@nestjs/graphql';
import { MongooseModule } from '@nestjs/mongoose';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { FilesService } from './files.service';
import { GeneralResolver } from './general.resolver';
import { Thread, ThreadSchema } from './models/thread.schema';
import { MessagesModule } from './modules/messages/messages.module';
import { SignaturesModule } from './modules/signatures/signatures.module';
import { ThreadsModule } from './modules/threads/threads.module';

@Module({
  imports: [
    ConfigModule.forRoot(),
    GraphQLModule.forRoot<ApolloDriverConfig>({
      driver: ApolloDriver,
      autoSchemaFile: 'src/schema.gql',
      installSubscriptionHandlers: true,
      subscriptions: {
        'graphql-ws': true,
      },
    }),
    MongooseModule.forFeature([{ name: Thread.name, schema: ThreadSchema }]),
    MongooseModule.forRoot(process.env.MONGO_URI),
    SignaturesModule,
    ThreadsModule,
    MessagesModule,
  ],
  controllers: [AppController],
  providers: [FilesService, AppService, GeneralResolver],
})
export class AppModule {}

import dotenv from 'dotenv';
dotenv.config();

import express, { Request, Response } from 'express';
import { initializeMongoDB } from './db/mongodb';
import nsselectionRouter from './routers/nnssf-nsselection';

const app = express();
const PORT = process.env.PORT || 3000;

app.use(express.json());

app.get('/health', (req: Request, res: Response) => {
  res.json({ status: 'ok' });
});

app.use('/nnssf-nsselection/v2', nsselectionRouter);

const startServer = async () => {
  try {
    await initializeMongoDB();
    console.log('MongoDB connected successfully');

    app.listen(PORT, () => {
      console.log(`NSSF server is running on port ${PORT}`);
    });
  } catch (error) {
    console.error('Failed to connect to MongoDB:', error);
    process.exit(1);
  }
};

startServer();

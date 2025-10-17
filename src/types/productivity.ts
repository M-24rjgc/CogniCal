export interface ProductivityScoreRecord {
  snapshotDate: string;
  compositeScore: number;
  dimensionScores: ProductivityDimensionScores;
  weightBreakdown: ProductivityDimensionWeights;
  explanation?: string;
  createdAt: string;
}

export interface ProductivityDimensionScores {
  completionRate: number;
  onTimeRatio: number;
  focusConsistency: number;
  restBalance: number;
  efficiencyRating: number;
}

export interface ProductivityDimensionWeights {
  completionRate: number;
  onTimeRatio: number;
  focusConsistency: number;
  restBalance: number;
  efficiencyRating: number;
}

export interface ProductivityScoreHistoryResponse {
  scores: ProductivityScoreRecord[];
  startDate: string;
  endDate: string;
  totalScores: number;
}

export interface ProductivityScoreUpsert {
  snapshotDate: string;
  compositeScore: number;
  dimensionScores: ProductivityDimensionScores;
  weightBreakdown: ProductivityDimensionWeights;
  explanation?: string;
}

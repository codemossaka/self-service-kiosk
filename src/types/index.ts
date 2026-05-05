export interface Sermon {
  code:     string
  title:    string
  date:     string
  year:     number
  filename: string
  lieu:     string
}

export interface Config {
  source:     string | null   // chemin local OU url https://
  sourceType: 'local' | 'remote' | null
}

export type Screen =
  | 's-setup'
  | 's-home'
  | 's-results'
  | 's-years'
  | 's-yr-list'
  | 's-preview'

export interface PrintResult {
  success: boolean
  reason:  string | null
}
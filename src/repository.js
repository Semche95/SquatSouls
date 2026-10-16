// Pages of the GitHub repository written in package.json. src-tauri/capabilities/default.json only allows
// opening these URLs: when repository.url changes in package.json, change them there too.
import { repository } from '../package.json';

export const repositoryUrl = repository.url;
export const releaseUrl = `${repository.url}/releases/latest`;
export const issuesUrl = `${repository.url}/issues/new`;

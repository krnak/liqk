import AsyncStorage from '@react-native-async-storage/async-storage';
import { v4 as uuidv4 } from 'uuid';

const TOKEN_KEY = 'lkd_access_token';
const BASE_URL_KEY = 'lkd_base_url';
const DEFAULT_BASE_URL = 'http://localhost:8080';

const PREFIXES = `
PREFIX posix: <http://www.w3.org/ns/posix/stat#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX liqk: <http://liqk.org/schema#>
PREFIX dc: <http://purl.org/dc/terms/>
PREFIX dcterms: <http://purl.org/dc/terms/>
`;

const FS_GRAPH = '<http://liqk.org/graph/filesystem>';
const KAIROS_GRAPH = '<http://liqk.org/graph/kairos>';

/**
 * LKD (Liqk Knowledge Database) service
 * Handles authentication and API requests to the oxigraph-gate proxy
 */
class LKDService {
  constructor() {
    this.accessToken = null;
    this.baseUrl = DEFAULT_BASE_URL;
  }

  async init() {
    const [token, url] = await Promise.all([
      AsyncStorage.getItem(TOKEN_KEY),
      AsyncStorage.getItem(BASE_URL_KEY),
    ]);
    this.accessToken = token;
    this.baseUrl = url || DEFAULT_BASE_URL;
    return this.hasToken();
  }

  hasToken() {
    return !!this.accessToken;
  }

  async setToken(token) {
    this.accessToken = token;
    await AsyncStorage.setItem(TOKEN_KEY, token);
  }

  async setBaseUrl(url) {
    this.baseUrl = url;
    await AsyncStorage.setItem(BASE_URL_KEY, url);
  }

  async clearToken() {
    this.accessToken = null;
    await AsyncStorage.removeItem(TOKEN_KEY);
  }

  getToken() {
    return this.accessToken;
  }

  getBaseUrl() {
    return this.baseUrl;
  }

  /**
   * Validate token by making a test request to the gate
   */
  async validateToken(token, baseUrl = this.baseUrl) {
    try {
      const response = await fetch(`${baseUrl}/query?query=ASK { ?s ?p ?o }`, {
        method: 'GET',
        headers: {
          'X-Access-Token': token,
        },
      });
      return response.ok;
    } catch (error) {
      console.error('Token validation error:', error);
      return false;
    }
  }

  /**
   * Make an authenticated request to LKD
   */
  async request(endpoint, options = {}) {
    if (!this.accessToken) {
      throw new Error('No access token configured');
    }

    const url = `${this.baseUrl}${endpoint}`;
    const headers = {
      'X-Access-Token': this.accessToken,
      ...options.headers,
    };

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = new Error(`LKD request failed: ${response.status}`);
      error.status = response.status;
      throw error;
    }

    return response;
  }

  /**
   * Execute a SPARQL query
   */
  async query(sparql) {
    const response = await this.request(
      `/query?query=${encodeURIComponent(sparql)}`,
      {
        method: 'GET',
        headers: {
          Accept: 'application/sparql-results+json',
        },
      }
    );
    return response.json();
  }

  /**
   * Execute a SPARQL update
   */
  async update(sparql) {
    const response = await this.request('/update', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/sparql-update',
      },
      body: sparql,
    });
    return response;
  }

  /**
   * List contents of a directory by path
   * @param {string} path - Directory path (e.g., "/" or "/upload")
   * @returns {Promise<Array>} Array of {uri, label, type, mimeType?, size?}
   */
  async listDirectory(path = '/') {
    const segments = path.split('/').filter(Boolean);

    let sparql;
    if (segments.length === 0) {
      // Root directory
      sparql = `${PREFIXES}
        SELECT ?item ?label ?type ?mimeType ?size FROM ${FS_GRAPH} WHERE {
          ?root a posix:Directory ;
                rdfs:label "/" ;
                posix:includes ?item .
          ?item rdfs:label ?label .
          ?item a ?type .
          FILTER(?type IN (posix:Directory, posix:File))
          OPTIONAL { ?item dc:format ?mimeType }
          OPTIONAL { ?item posix:size ?size }
        }
        ORDER BY ?type ?label`;
    } else {
      // Build traversal for nested path
      let traversal = `?root a posix:Directory ; rdfs:label "/" .\n`;
      for (let i = 0; i < segments.length; i++) {
        const prev = i === 0 ? '?root' : `?dir${i - 1}`;
        const curr = `?dir${i}`;
        traversal += `${prev} posix:includes ${curr} .\n`;
        traversal += `${curr} rdfs:label "${segments[i]}" .\n`;
      }
      const lastDir = `?dir${segments.length - 1}`;

      sparql = `${PREFIXES}
        SELECT ?item ?label ?type ?mimeType ?size FROM ${FS_GRAPH} WHERE {
          ${traversal}
          ${lastDir} posix:includes ?item .
          ?item rdfs:label ?label .
          ?item a ?type .
          FILTER(?type IN (posix:Directory, posix:File))
          OPTIONAL { ?item dc:format ?mimeType }
          OPTIONAL { ?item posix:size ?size }
        }
        ORDER BY ?type ?label`;
    }

    const result = await this.query(sparql);
    return result.results.bindings.map(b => ({
      uri: b.item.value,
      label: b.label.value,
      type: b.type.value.includes('Directory') ? 'directory' : 'file',
      mimeType: b.mimeType?.value,
      size: b.size?.value ? parseInt(b.size.value, 10) : undefined,
    }));
  }

  /**
   * Get file content by path
   * @param {string} path - File path
   * @returns {Promise<string>} File content as text
   */
  async getFileContent(path) {
    const response = await this.request(`/file${path}`);
    return response.text();
  }

  /**
   * Get file URL for direct access
   * @param {string} path - File path
   * @returns {string} Full URL with auth
   */
  getFileUrl(path) {
    return `${this.baseUrl}/file${path}`;
  }

  /**
   * Get priority tasks (priority >= pinned, not completed/cancelled)
   * @returns {Promise<Array>} Array of tasks ordered by priority desc
   */
  async getPriorityTasks() {
    const sparql = `${PREFIXES}
      SELECT ?task ?title ?priority ?priorityRank ?status ?projectTitle ?readme FROM ${KAIROS_GRAPH} WHERE {
        ?task a liqk:Task ;
              liqk:title ?title ;
              liqk:priority ?priority .
        ?priority liqk:priority-rank ?priorityRank .
        liqk:priority-pinned liqk:priority-rank ?pinnedRank .
        FILTER(?priorityRank >= ?pinnedRank)
        OPTIONAL { ?task liqk:task-status ?status }
        OPTIONAL { ?task liqk:project ?project . ?project liqk:title ?projectTitle }
        OPTIONAL { ?task liqk:readme ?readme }
        FILTER(!BOUND(?status) || (?status != liqk:task-status-done &&
               ?status != liqk:task-status-hall-of-fame &&
               ?status != liqk:task-status-trashed))
      }
      ORDER BY DESC(?priorityRank)`;

    const result = await this.query(sparql);
    return this._mapTasks(result);
  }

  /**
   * Get all tasks
   * @returns {Promise<Array>} Array of all tasks ordered by priority desc
   */
  async getAllTasks() {
    const sparql = `${PREFIXES}
      SELECT ?task ?title ?priority ?priorityRank ?status ?projectTitle ?readme FROM ${KAIROS_GRAPH} WHERE {
        ?task a liqk:Task ;
              liqk:title ?title .
        OPTIONAL { ?task liqk:priority ?priority . ?priority liqk:priority-rank ?priorityRank }
        OPTIONAL { ?task liqk:task-status ?status }
        OPTIONAL { ?task liqk:project ?project . ?project liqk:title ?projectTitle }
        OPTIONAL { ?task liqk:readme ?readme }
      }
      ORDER BY DESC(?priorityRank)`;

    const result = await this.query(sparql);
    return this._mapTasks(result);
  }

  /**
   * Get completed tasks
   * @returns {Promise<Array>} Array of completed tasks
   */
  async getCompletedTasks() {
    const sparql = `${PREFIXES}
      SELECT ?task ?title ?priority ?priorityRank ?status ?projectTitle ?readme FROM ${KAIROS_GRAPH} WHERE {
        ?task a liqk:Task ;
              liqk:title ?title ;
              liqk:task-status ?status .
        FILTER(?status = liqk:task-status-done || ?status = liqk:task-status-hall-of-fame)
        OPTIONAL { ?task liqk:priority ?priority . ?priority liqk:priority-rank ?priorityRank }
        OPTIONAL { ?task liqk:project ?project . ?project liqk:title ?projectTitle }
        OPTIONAL { ?task liqk:readme ?readme }
      }
      ORDER BY DESC(?priorityRank)`;

    const result = await this.query(sparql);
    return this._mapTasks(result);
  }

  _mapTasks(result) {
    return result.results.bindings.map(b => ({
      uri: b.task.value,
      title: b.title.value,
      priority: b.priority?.value?.split('#')[1],
      priorityRank: b.priorityRank?.value ? parseInt(b.priorityRank.value, 10) : 0,
      status: b.status?.value?.split('#')[1],
      project: b.projectTitle?.value,
      readmeUri: b.readme?.value,
      readmeUuid: b.readme?.value?.replace('urn:uuid:', ''),
    }));
  }

  /**
   * Update task status and create ModifyAction
   * @param {string} taskUri - Task URI
   * @param {string} newStatus - New status (e.g., 'task-status-done', 'task-status-trashed')
   */
  async updateTaskStatus(taskUri, newStatus) {
    // First get current status
    const getStatusSparql = `${PREFIXES}
      SELECT ?status FROM ${KAIROS_GRAPH} WHERE {
        <${taskUri}> liqk:task-status ?status .
      }`;

    const result = await this.query(getStatusSparql);
    const oldStatus = result.results.bindings[0]?.status?.value;

    const actionUri = `urn:uuid:${uuidv4()}`;
    const timestamp = Math.floor(Date.now() / 1000);
    const newStatusUri = `liqk:${newStatus}`;
    const oldStatusValue = oldStatus ? `<${oldStatus}>` : 'liqk:task-status-not-started';

    let sparql;
    if (oldStatus) {
      // Update existing status
      sparql = `${PREFIXES}
        DELETE DATA {
          GRAPH ${KAIROS_GRAPH} {
            <${taskUri}> liqk:task-status <${oldStatus}> .
          }
        };
        INSERT DATA {
          GRAPH ${KAIROS_GRAPH} {
            <${taskUri}> liqk:task-status ${newStatusUri} .
            <${actionUri}> a liqk:ModifyAction ;
              dcterms:subject <${taskUri}> ;
              liqk:modified-property liqk:task-status ;
              liqk:old-value ${oldStatusValue} ;
              liqk:new-value ${newStatusUri} ;
              dc:created ${timestamp} .
          }
        }`;
    } else {
      // Insert new status
      sparql = `${PREFIXES}
        INSERT DATA {
          GRAPH ${KAIROS_GRAPH} {
            <${taskUri}> liqk:task-status ${newStatusUri} .
            <${actionUri}> a liqk:ModifyAction ;
              dcterms:subject <${taskUri}> ;
              liqk:modified-property liqk:task-status ;
              liqk:old-value liqk:task-status-not-started ;
              liqk:new-value ${newStatusUri} ;
              dc:created ${timestamp} .
          }
        }`;
    }

    await this.update(sparql);
  }

  /**
   * Upload a markdown file
   * @param {string} filename - Original filename
   * @param {string} content - File content
   * @returns {Promise<string>} File UUID
   */
  async uploadMarkdown(filename, content) {
    if (!this.accessToken) {
      throw new Error('No access token configured');
    }

    const formData = new FormData();
    const blob = new Blob([content], { type: 'text/markdown' });
    formData.append('file', blob, filename);

    const response = await fetch(`${this.baseUrl}/upload`, {
      method: 'POST',
      headers: {
        'X-Access-Token': this.accessToken,
        'Accept': 'application/json',
      },
      body: formData,
    });

    if (!response.ok) {
      throw new Error(`Upload failed: ${response.status}`);
    }

    // Parse JSON response to get file UUID
    const result = await response.json();
    return result.uuid;
  }

  /**
   * Get file content by UUID
   * @param {string} uuid - File UUID
   * @returns {Promise<string>} File content
   */
  async getFileByUuid(uuid) {
    const response = await this.request(`/res/${uuid}`);
    return response.text();
  }

  /**
   * Get all abbreviations (projects and priorities)
   * @returns {Promise<{projects: Array, priorities: Array}>}
   */
  async getAbbreviations() {
    const sparql = `${PREFIXES}
      SELECT ?resource ?abbrv ?title ?type FROM ${KAIROS_GRAPH} WHERE {
        ?resource liqk:abbrv ?abbrv .
        OPTIONAL { ?resource liqk:title ?title }
        BIND(
          IF(STRSTARTS(STR(?resource), "http://liqk.org/schema#priority-"), "priority", "project")
          AS ?type
        )
      }`;

    const result = await this.query(sparql);
    const projects = [];
    const priorities = [];

    for (const b of result.results.bindings) {
      const item = {
        uri: b.resource.value,
        abbrv: b.abbrv.value,
        title: b.title?.value,
      };
      if (b.type.value === 'priority') {
        item.name = b.resource.value.split('#')[1];
        priorities.push(item);
      } else {
        projects.push(item);
      }
    }

    return { projects, priorities };
  }

  /**
   * Create a new task
   * @param {string} title - Task title
   * @param {string|null} priorityUri - Priority URI or null
   * @param {Array<string>} projectUris - Array of project URIs
   * @param {string|null} readmeContent - Optional markdown content for readme
   */
  async createTask(title, priorityUri, projectUris = [], readmeContent = null) {
    const taskUri = `urn:uuid:${uuidv4()}`;
    let readmeUri = null;

    // Upload readme file if content provided
    if (readmeContent) {
      const fileUuid = await this.uploadMarkdown(`${title}.md`, readmeContent);
      readmeUri = `urn:uuid:${fileUuid}`;
    }

    let triples = `<${taskUri}> a liqk:Task ;\n      liqk:title "${title.replace(/"/g, '\\"')}" ;\n      liqk:task-status liqk:task-status-not-started `;

    if (priorityUri) {
      triples += `;\n      liqk:priority <${priorityUri}> `;
    }

    for (const projectUri of projectUris) {
      triples += `;\n      liqk:project <${projectUri}> `;
    }

    if (readmeUri) {
      triples += `;\n      liqk:readme <${readmeUri}> `;
    }

    triples += '.';

    const sparql = `${PREFIXES}
      INSERT DATA {
        GRAPH ${KAIROS_GRAPH} {
          ${triples}
        }
      }`;

    await this.update(sparql);
    return taskUri;
  }

  /**
   * Get all ModifyActions ordered by newest first
   * @returns {Promise<Array>} Array of actions
   */
  async getActions() {
    const sparql = `${PREFIXES}
      SELECT ?action ?subject ?subjectTitle ?property ?oldValue ?newValue ?created
      FROM ${KAIROS_GRAPH} WHERE {
        ?action a liqk:ModifyAction ;
                dcterms:subject ?subject ;
                liqk:modified-property ?property ;
                liqk:new-value ?newValue ;
                dc:created ?created .
        OPTIONAL { ?action liqk:old-value ?oldValue }
        OPTIONAL { ?subject liqk:title ?subjectTitle }
      }
      ORDER BY DESC(?created)`;

    const result = await this.query(sparql);
    return result.results.bindings.map(b => ({
      uri: b.action.value,
      subjectUri: b.subject.value,
      subjectTitle: b.subjectTitle?.value,
      property: b.property.value.split('#')[1],
      oldValue: b.oldValue?.value?.split('#')[1],
      newValue: b.newValue.value.split('#')[1],
      created: parseInt(b.created.value, 10),
    }));
  }
}

export const lkd = new LKDService();
export default lkd;

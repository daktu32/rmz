"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PromptSelector = void 0;
const prompts_1 = require("@inquirer/prompts");
class PromptSelector {
    static async selectPrompt() {
        console.log('\n🎯 Prompt Selection - Choose the development approach that fits your project\n');
        try {
            const teamSize = await (0, prompts_1.input)({
                message: 'How many developers will work on this project?',
                default: '1',
                validate: (value) => {
                    const num = parseInt(value, 10);
                    return !isNaN(num) && num > 0 ? true : 'Team size must be a number greater than 0';
                },
            });
            const industry = await (0, prompts_1.select)({
                message: 'What industry/domain is this project for?',
                choices: [
                    { name: 'Technology/Software', value: 'technology' },
                    { name: 'Finance/Banking', value: 'finance' },
                    { name: 'Healthcare', value: 'healthcare' },
                    { name: 'Education', value: 'education' },
                    { name: 'E-commerce/Retail', value: 'ecommerce' },
                    { name: 'Government/Public', value: 'government' },
                    { name: 'Entertainment/Media', value: 'entertainment' },
                    { name: 'Other', value: 'other' },
                ],
            });
            const projectType = await (0, prompts_1.select)({
                message: 'What type of project is this?',
                choices: [
                    { name: 'Personal/Learning project', value: 'personal' },
                    { name: 'Open source project', value: 'opensource' },
                    { name: 'Startup/MVP', value: 'startup' },
                    { name: 'Enterprise application', value: 'enterprise' },
                    { name: 'Client project', value: 'client' },
                ],
            });
            const complianceLevel = await (0, prompts_1.select)({
                message: 'What level of compliance/governance do you need?',
                choices: [
                    { name: 'Low - Minimal documentation, fast iteration', value: 'low' },
                    { name: 'Medium - Standard practices, moderate documentation', value: 'medium' },
                    { name: 'High - Strict compliance, comprehensive documentation', value: 'high' },
                ],
            });
            const answers = {
                teamSize: parseInt(teamSize, 10),
                industry,
                projectType,
                complianceLevel,
            };
            const team = {
                size: answers.teamSize,
                type: this.getTeamType(answers.teamSize),
                industry: answers.industry,
                complianceLevel: answers.complianceLevel,
            };
            const recommendedPrompt = this.recommendPrompt(team, answers.projectType);
            console.log(`\n💡 Recommended prompt: ${recommendedPrompt}`);
            console.log(this.getPromptDescription(recommendedPrompt));
            const useRecommended = await (0, prompts_1.confirm)({
                message: 'Use the recommended prompt?',
                default: true,
            });
            let selectedPrompt = recommendedPrompt;
            if (!useRecommended) {
                selectedPrompt = await (0, prompts_1.select)({
                    message: 'Choose a different prompt:',
                    choices: [
                        {
                            name: 'Basic Development - Small teams, simple workflow',
                            value: 'basic-development',
                        },
                        {
                            name: 'Enterprise Development - Large teams, compliance focus',
                            value: 'enterprise-development',
                        },
                        {
                            name: 'Open Source Development - Community-driven projects',
                            value: 'opensource-development',
                        },
                        {
                            name: 'Startup Development - Fast iteration, MVP focus',
                            value: 'startup-development',
                        },
                    ],
                });
            }
            return { prompt: selectedPrompt, team };
        }
        catch (error) {
            console.error('Error during prompt selection:', error);
            throw new Error('Failed to select prompt configuration');
        }
    }
    static getTeamType(size) {
        if (size === 1)
            return 'individual';
        if (size <= 3)
            return 'small';
        if (size <= 10)
            return 'medium';
        return 'large';
    }
    static recommendPrompt(team, projectType) {
        if (team.complianceLevel === 'high' || team.size > 10) {
            return 'enterprise-development';
        }
        if (projectType === 'opensource') {
            return 'opensource-development';
        }
        if (projectType === 'startup' || projectType === 'personal') {
            return 'startup-development';
        }
        return 'basic-development';
    }
    static getPromptDescription(prompt) {
        const descriptions = {
            'basic-development': '  → Perfect for small teams (1-3 developers) with straightforward workflow needs',
            'enterprise-development': '  → Designed for large teams with compliance requirements and complex governance',
            'opensource-development': '  → Optimized for community-driven projects with contributor management',
            'startup-development': '  → Focused on rapid iteration, MVP development, and fast time-to-market',
        };
        return descriptions[prompt];
    }
}
exports.PromptSelector = PromptSelector;

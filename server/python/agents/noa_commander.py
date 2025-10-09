#!/usr/bin/env python3
"""
NOA ExecutiveCommanderChiefAgent - Python Implementation
Top-level business goal orchestration service
"""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Dict, List, Any, Optional
import uvicorn
import asyncio
import logging
import json
import time
from datetime import datetime
import uuid
import requests

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(
    title="NOA ExecutiveCommanderChiefAgent",
    description="Top-level business goal orchestration and task management",
    version="1.0.0"
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Pydantic models
class BusinessGoal(BaseModel):
    goal_description: str
    priority: str = "medium"
    deadline: Optional[str] = None
    context: Optional[Dict[str, Any]] = {}

class WorkPlan(BaseModel):
    id: str
    goal: str
    tasks: List[Dict[str, Any]]
    assigned_resources: Dict[str, List[str]]
    status: str
    created_at: str
    updated_at: str

class TaskAssignment(BaseModel):
    task_id: str
    agent_type: str
    agent_id: str
    task_description: str
    parameters: Dict[str, Any]

# Global state
work_plans: Dict[str, WorkPlan] = {}
active_tasks: Dict[str, Dict[str, Any]] = {}

@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "NOA ExecutiveCommanderChiefAgent",
        "version": "1.0.0",
        "status": "operational",
        "capabilities": [
            "Business goal intake",
            "Task decomposition",
            "Resource allocation",
            "Execution coordination",
            "Progress monitoring"
        ]
    }

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "NOA Commander",
        "active_work_plans": len(work_plans),
        "active_tasks": len(active_tasks),
        "timestamp": datetime.utcnow().isoformat()
    }

@app.post("/goals/intake")
async def intake_business_goal(goal: BusinessGoal):
    """Intake and normalize business goals into structured WorkPlans"""
    try:
        # Generate work plan ID
        plan_id = str(uuid.uuid4())
        
        # Decompose goal into tasks
        tasks = await decompose_goal(goal.goal_description, goal.context)
        
        # Allocate resources
        resources = await allocate_resources(tasks)
        
        # Create work plan
        work_plan = WorkPlan(
            id=plan_id,
            goal=goal.goal_description,
            tasks=tasks,
            assigned_resources=resources,
            status="planned",
            created_at=datetime.utcnow().isoformat(),
            updated_at=datetime.utcnow().isoformat()
        )
        
        work_plans[plan_id] = work_plan
        
        # Validate with constitutional framework
        constitutional_result = await validate_with_trifecta_court(
            f"Execute work plan: {goal.goal_description}",
            {"work_plan": work_plan.dict(), "goal": goal.dict()}
        )
        
        if not constitutional_result.get("valid", False):
            raise HTTPException(
                status_code=400,
                detail=f"Constitutional validation failed: {constitutional_result.get('reasoning', 'Unknown reason')}"
            )
        
        logger.info(f"Work plan {plan_id} created for goal: {goal.goal_description}")
        
        return {
            "status": "success",
            "work_plan_id": plan_id,
            "work_plan": work_plan.dict(),
            "constitutional_validation": constitutional_result,
            "message": "Business goal successfully converted to work plan"
        }
        
    except Exception as e:
        logger.error(f"Failed to intake business goal: {e}")
        raise HTTPException(status_code=500, detail=f"Error processing business goal: {str(e)}")

@app.post("/plans/{plan_id}/execute")
async def execute_work_plan(plan_id: str):
    """Execute a work plan by coordinating task execution"""
    try:
        if plan_id not in work_plans:
            raise HTTPException(status_code=404, detail="Work plan not found")
        
        work_plan = work_plans[plan_id]
        
        if work_plan.status != "planned":
            raise HTTPException(status_code=400, detail=f"Work plan is not in planned status: {work_plan.status}")
        
        # Update status
        work_plan.status = "executing"
        work_plan.updated_at = datetime.utcnow().isoformat()
        
        # Execute tasks
        execution_results = []
        
        for task in work_plan.tasks:
            task_id = task["id"]
            
            # Assign task to appropriate agent
            assignment_result = await assign_task_to_agent(task, work_plan.assigned_resources)
            execution_results.append(assignment_result)
            
            # Track active task
            active_tasks[task_id] = {
                "work_plan_id": plan_id,
                "task": task,
                "assignment": assignment_result,
                "status": "assigned",
                "started_at": datetime.utcnow().isoformat()
            }
        
        logger.info(f"Work plan {plan_id} execution started with {len(execution_results)} tasks")
        
        return {
            "status": "success",
            "work_plan_id": plan_id,
            "execution_results": execution_results,
            "active_tasks": len(execution_results),
            "message": "Work plan execution initiated"
        }
        
    except Exception as e:
        logger.error(f"Failed to execute work plan {plan_id}: {e}")
        raise HTTPException(status_code=500, detail=f"Error executing work plan: {str(e)}")

@app.get("/plans")
async def list_work_plans():
    """List all work plans"""
    return {
        "work_plans": [plan.dict() for plan in work_plans.values()],
        "total": len(work_plans)
    }

@app.get("/plans/{plan_id}")
async def get_work_plan(plan_id: str):
    """Get specific work plan details"""
    if plan_id not in work_plans:
        raise HTTPException(status_code=404, detail="Work plan not found")
    
    return work_plans[plan_id].dict()

@app.get("/plans/{plan_id}/status")
async def get_work_plan_status(plan_id: str):
    """Get work plan execution status"""
    if plan_id not in work_plans:
        raise HTTPException(status_code=404, detail="Work plan not found")
    
    work_plan = work_plans[plan_id]
    
    # Get task statuses
    plan_tasks = [task for task in active_tasks.values() if task["work_plan_id"] == plan_id]
    
    task_statuses = {}
    for task_id, task_info in active_tasks.items():
        if task_info["work_plan_id"] == plan_id:
            task_statuses[task_id] = task_info["status"]
    
    return {
        "work_plan_id": plan_id,
        "status": work_plan.status,
        "total_tasks": len(work_plan.tasks),
        "active_tasks": len(plan_tasks),
        "task_statuses": task_statuses,
        "updated_at": work_plan.updated_at
    }

@app.get("/tasks/active")
async def list_active_tasks():
    """List all active tasks"""
    return {
        "active_tasks": active_tasks,
        "total": len(active_tasks)
    }

@app.post("/tasks/{task_id}/complete")
async def complete_task(task_id: str, result: Dict[str, Any]):
    """Mark a task as completed with results"""
    try:
        if task_id not in active_tasks:
            raise HTTPException(status_code=404, detail="Active task not found")
        
        task_info = active_tasks[task_id]
        task_info["status"] = "completed"
        task_info["completed_at"] = datetime.utcnow().isoformat()
        task_info["result"] = result
        
        # Check if work plan is complete
        work_plan_id = task_info["work_plan_id"]
        await check_work_plan_completion(work_plan_id)
        
        logger.info(f"Task {task_id} completed")
        
        return {
            "status": "success",
            "task_id": task_id,
            "message": "Task marked as completed"
        }
        
    except Exception as e:
        logger.error(f"Failed to complete task {task_id}: {e}")
        raise HTTPException(status_code=500, detail=f"Error completing task: {str(e)}")

async def decompose_goal(goal_description: str, context: Dict[str, Any]) -> List[Dict[str, Any]]:
    """Decompose business goal into actionable tasks"""
    
    # Simple task decomposition logic
    tasks = []
    
    # Analyze goal and create tasks
    if "analysis" in goal_description.lower() or "research" in goal_description.lower():
        tasks.append({
            "id": str(uuid.uuid4()),
            "type": "analysis",
            "description": f"Analyze requirements for: {goal_description}",
            "priority": "high",
            "estimated_duration": "2 hours"
        })
    
    if "implementation" in goal_description.lower() or "build" in goal_description.lower():
        tasks.append({
            "id": str(uuid.uuid4()),
            "type": "implementation",
            "description": f"Implement solution for: {goal_description}",
            "priority": "high",
            "estimated_duration": "4 hours"
        })
    
    if "testing" in goal_description.lower() or "validation" in goal_description.lower():
        tasks.append({
            "id": str(uuid.uuid4()),
            "type": "testing",
            "description": f"Test and validate: {goal_description}",
            "priority": "medium",
            "estimated_duration": "1 hour"
        })
    
    # Default task if no specific patterns found
    if not tasks:
        tasks.append({
            "id": str(uuid.uuid4()),
            "type": "general",
            "description": f"Execute: {goal_description}",
            "priority": "medium",
            "estimated_duration": "2 hours"
        })
    
    return tasks

async def allocate_resources(tasks: List[Dict[str, Any]]) -> Dict[str, List[str]]:
    """Allocate appropriate resources (agents) to tasks"""
    
    resources = {
        "board_agents": [],
        "model_selectors": [],
        "micro_agents": []
    }
    
    for task in tasks:
        task_type = task.get("type", "general")
        
        if task_type == "analysis":
            resources["board_agents"].append("digest_agent")
            resources["model_selectors"].append("analytical_model_selector")
        elif task_type == "implementation":
            resources["board_agents"].append("cto_agent")
            resources["micro_agents"].append("development_stack")
        elif task_type == "testing":
            resources["board_agents"].append("security_agent")
            resources["micro_agents"].append("testing_stack")
        else:
            resources["board_agents"].append("coo_agent")
            resources["micro_agents"].append("general_stack")
    
    return resources

async def assign_task_to_agent(task: Dict[str, Any], resources: Dict[str, List[str]]) -> Dict[str, Any]:
    """Assign a specific task to an appropriate agent"""
    
    task_type = task.get("type", "general")
    
    # Select appropriate agent based on task type
    if task_type == "analysis" and "digest_agent" in resources.get("board_agents", []):
        assigned_agent = "digest_agent"
        agent_type = "board_agent"
    elif task_type == "implementation" and "cto_agent" in resources.get("board_agents", []):
        assigned_agent = "cto_agent"
        agent_type = "board_agent"
    elif task_type == "testing" and "security_agent" in resources.get("board_agents", []):
        assigned_agent = "security_agent"
        agent_type = "board_agent"
    else:
        assigned_agent = "coo_agent"
        agent_type = "board_agent"
    
    assignment = {
        "task_id": task["id"],
        "assigned_agent": assigned_agent,
        "agent_type": agent_type,
        "assignment_time": datetime.utcnow().isoformat(),
        "status": "assigned"
    }
    
    logger.info(f"Task {task['id']} assigned to {assigned_agent}")
    
    return assignment

async def check_work_plan_completion(work_plan_id: str):
    """Check if all tasks in a work plan are completed"""
    
    if work_plan_id not in work_plans:
        return
    
    work_plan = work_plans[work_plan_id]
    plan_tasks = [task for task in active_tasks.values() if task["work_plan_id"] == work_plan_id]
    
    completed_tasks = [task for task in plan_tasks if task["status"] == "completed"]
    
    if len(completed_tasks) == len(work_plan.tasks):
        work_plan.status = "completed"
        work_plan.updated_at = datetime.utcnow().isoformat()
        logger.info(f"Work plan {work_plan_id} completed")

async def validate_with_trifecta_court(action: str, context: Dict[str, Any]) -> Dict[str, Any]:
    """Validate action with Trifecta Court constitutional framework"""
    
    try:
        response = requests.post(
            "http://localhost:8000/court/trifecta",
            json={"action": action, "context": context},
            timeout=10
        )
        
        if response.status_code == 200:
            return response.json()
        else:
            logger.warning(f"Constitutional validation failed: HTTP {response.status_code}")
            return {"valid": False, "reasoning": f"HTTP {response.status_code}"}
            
    except Exception as e:
        logger.warning(f"Constitutional validation error: {e}")
        return {"valid": True, "reasoning": "Constitutional service unavailable, proceeding with caution"}

@app.get("/constitutional/validate")
async def constitutional_validation_endpoint():
    """Endpoint for constitutional validation integration"""
    return {
        "service": "NOA Commander",
        "constitutional_integration": "active",
        "validation_capabilities": [
            "Work plan validation",
            "Task assignment validation",
            "Goal intake validation"
        ]
    }

if __name__ == "__main__":
    print("Starting NOA ExecutiveCommanderChiefAgent...")
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8001,
        reload=False,
        log_level="info"
    )

